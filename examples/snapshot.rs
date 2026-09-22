//! Fingerprints real splitter output, so a dependency bump can be shown to change nothing.
//!
//! The unit tests assert *self-consistent* invariants — splits tile the input, their tokens
//! partition the document encoding — which hold whatever the tokenizer or the pattern
//! matcher does. They would stay green even if the actual boundaries moved. This prints a
//! hash of the real output instead, so a change is visible:
//!
//! ```text
//! cargo run --release --example snapshot > /tmp/before.txt
//! # ...edit Cargo.toml, cargo update -p <crate>...
//! cargo run --release --example snapshot > /tmp/after.txt
//! diff /tmp/before.txt /tmp/after.txt        # empty means nothing observable changed
//! ```
//!
//! Runs single-threaded so the output is deterministic. Needs the HF models in the hub
//! cache (or network on first run) and the corpora under `tests/`.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};

use aho_corasick::Span;
use fast_text_splitter::config::SplitterLiteConfig;
use fast_text_splitter::encodings::NoneTokenizer;
use fast_text_splitter::hf_tokenizer::init_tokenizer;
use fast_text_splitter::pattern_search::pattern_searcher::PatternSearcher;
use fast_text_splitter::splitter::split_node::visualization::term_tree;
use fast_text_splitter::splitter::splitter_config::SplitterConfig;
use fast_text_splitter::splitter::Splitter;

const EN: &str = "sentence-transformers/all-MiniLM-L6-v2";
const ML: &str = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2";

fn hash<T: Hash>(v: &T) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

fn pats(levels: &[&[&str]]) -> Vec<Vec<String>> {
    levels
        .iter()
        .map(|l| l.iter().map(|p| p.to_string()).collect())
        .collect()
}

/// Ids and offsets the tokenizer itself produces, before any splitting.
fn tokenizer_fingerprints() {
    let text = "Dr. Smith went to Washington.\nLächeln ist die kürzeste Entfernung. 这是第一句。";
    for (name, model) in [("en", EN), ("ml", ML)] {
        let tk = init_tokenizer(Some(model.to_string()), Some(usize::MAX), false).unwrap();
        let e = tk.encode(text, false).unwrap();
        println!(
            "tokenizer {name:<3} len={:<4} ids={:016x} offsets={:016x}",
            e.len(),
            hash(&e.get_ids().to_vec()),
            hash(&e.get_offsets().to_vec())
        );
    }
}

/// The rendered debug tree, which is what `termtree` controls.
fn tree_fingerprint() {
    let data = fs::read("tests/test_data/en_long_paragraphs.txt").unwrap();
    let patterns = pats(&[&["\n\n"], &["\n"], &[".", "!", "?"]]);
    let searchers: Vec<_> = patterns
        .iter()
        .map(|p| PatternSearcher::new(p.clone()))
        .collect();
    let config = SplitterConfig::<NoneTokenizer> {
        data: &data,
        searchers: &searchers,
        tokenizer: None,
        max_len: Some(256),
        merge_level: None,
        patterns_len: searchers.len(),
    };
    let span = Span {
        start: 0,
        end: data.len(),
    };
    let tree = Splitter::new(&config, span, 0, span, None, None, None).split();
    let rendered = format!("{}", term_tree(&tree, &data).unwrap());
    println!(
        "tree      lines={:<4} chars={:<6} render={:016x}",
        rendered.lines().count(),
        rendered.chars().count(),
        hash(&rendered)
    );
}

/// End-to-end splits over the committed corpora.
fn split_fingerprints() {
    let cases: Vec<(&str, Vec<Vec<String>>, usize, &str)> = vec![
        (
            "tests/test_data/superlinear.txt",
            pats(&[&["\n\n"], &["\n"], &[".", "!", "?"]]),
            512,
            EN,
        ),
        (
            "tests/test_data/superlinear.txt",
            pats(&[&["<SENT>"], &["\n\n"], &["\n"]]),
            60,
            EN,
        ),
        (
            "tests/test_data/superlinear.txt",
            pats(&[&["<ICU_SENT>"], &["\n\n"], &["\n"]]),
            512,
            EN,
        ),
        // 5 alternatives at one level: this is the path that uses AhoCorasick rather than memchr.
        (
            "tests/test_data/superlinear.txt",
            pats(&[&["\n\n"], &["\n"], &[". ", "! ", "? ", ", ", "; "]]),
            256,
            EN,
        ),
        (
            "tests/test_data/en_long_paragraphs.txt",
            pats(&[&["\n\n"], &["\n"], &[".", "!", "?"]]),
            128,
            EN,
        ),
        // No spaces anywhere: forces hard cuts inside a word.
        (
            "tests/test_data/newsroom_no_space.txt",
            pats(&[&["\n\n"], &["\n"], &[".", "!", "?"]]),
            64,
            EN,
        ),
        (
            "tests/test_data_code/text_splitters.py",
            pats(&[
                &["\nclass ", "\ndef "],
                &["\n\tdef ", "\n\n"],
                &["\n"],
                &[" "],
            ]),
            128,
            EN,
        ),
        (
            "tests/test_data/chinese_example01.txt",
            pats(&[&["\n\n"], &["\n"], &[".", ". ", ",", ", ", "、", "。"]]),
            512,
            ML,
        ),
        (
            "tests/test_data/vertragsgrundlagen_efh_wohn.txt",
            pats(&[&["\n\n"], &["\n"], &[".", "!", "?"]]),
            512,
            ML,
        ),
        (
            "tests/test_data/telekomturkishsample.txt",
            pats(&[&["\n\n"], &["\n"], &[".", "!", "?"]]),
            512,
            ML,
        ),
    ];
    for (path, patterns, max_tokens, model) in cases {
        let data = fs::read(path).unwrap();
        let level0 = patterns[0][0].replace('\n', "\\n");
        let widest = patterns.iter().map(|p| p.len()).max().unwrap_or(0);
        let cfg = SplitterLiteConfig::new_hf(
            patterns,
            Some(max_tokens),
            None,
            false,
            Some(model.to_string()),
        );
        let splits = cfg.hf_splits(&data);
        let texts: Vec<&str> = splits.iter().map(|s| s.split_string.as_str()).collect();
        let ids: Vec<u32> = splits.iter().flat_map(|s| s.tokens.clone()).collect();
        println!(
            "split {:<40} lvl0={:<11} widest={} max={:<4} n={:<5} tok={:<6} text={:016x} ids={:016x}",
            path.rsplit('/').next().unwrap(),
            level0,
            widest,
            max_tokens,
            splits.len(),
            ids.len(),
            hash(&texts),
            hash(&ids)
        );
    }
}

fn main() {
    tokenizer_fingerprints();
    tree_fingerprint();
    split_fingerprints();
}
