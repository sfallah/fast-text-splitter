//! Helpers shared by the unit tests: sample texts, pattern builders, cached
//! tokenizers, and the invariants every splitter result has to satisfy.

use std::fs;
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::LazyLock;

use aho_corasick::Span;
use tokenizers::Tokenizer;

use crate::config::SplitterLiteConfig;
use crate::encodings::Tokenize;
use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
use crate::pattern_search::pattern_searcher::PatternSearcher;
use crate::splitter::split_node::utils::SplitResultLite;
use crate::splitter::split_node::SplitNode;
use crate::splitter::splitter_config::SplitterConfig;
use crate::splitter::Splitter;

pub const DEFAULT_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";
pub const MULTILINGUAL_MODEL: &str = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2";

/// Paragraphs separated by runs of blank lines, some of them empty.
pub const PARAGRAPHS: &str = "\n\n\
    \n\n\
    \n\n\
    In fact, the correlation. Between superlinear.\n\
    Returns and inequality is so strong that it yields.\n\n\
    Another heuristic for.\n\n\
    \n\n\
    \n\n\
    \nFinding work of this type.\n\
    Look for fields where.\n\
    A few big winners. \n\
    Outperform everyone else.\n\n";

/// One sentence per line, no blank lines: the first pattern level never matches.
pub const LINES: &str = "\n\
    In fact, the correlation. Between superlinear.\n\
    Returns and inequality is so strong that it yields.\n\
    Another heuristic for.\n\
    Finding work of this type.\n\
    Look for fields where.\n\
    A few big winners. \n\
    Outperform everyone else.\n";

/// A leading separator, a stray single newline inside a blank-line run, and a trailing separator.
pub const SHORT: &str = "\n\n\
    Returns and inequality, is so strong.\n\
    That it yields.\n\n\
    Another heuristic for.\n\n\
    \n\
    Outperform everyone else.\n\n";

pub fn patterns(levels: &[&[&str]]) -> Vec<Vec<String>> {
    levels
        .iter()
        .map(|level| level.iter().map(|p| p.to_string()).collect())
        .collect()
}

/// Blank line, then newline, then sentence punctuation: the hierarchy most tests use.
pub fn paragraph_patterns() -> Vec<Vec<String>> {
    patterns(&[&["\n\n"], &["\n"], &[".", "!", "?"]])
}

/// A sentence segmenter (`<SENT>` or `<ICU_SENT>`) first, then blank line, then newline.
pub fn sentence_first_patterns(marker: &str) -> Vec<Vec<String>> {
    patterns(&[&[marker], &["\n\n"], &["\n"]])
}

pub fn searchers(patterns: &[Vec<String>]) -> Vec<PatternSearcher> {
    patterns
        .iter()
        .map(|p| PatternSearcher::new(p.clone()))
        .collect()
}

pub fn read(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| panic!("{path}: {err}"))
}

/// Sorted `.txt` files directly under `dir`.
pub fn text_files(dir: &str) -> Vec<String> {
    let mut files: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("{dir}: {err}"))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .map(|path| path.to_str().unwrap().to_string())
        .collect();
    files.sort();
    files
}

// `Tokenizer::from_pretrained` races on hf-hub's cache lock when parallel tests load the
// same model, so each model is loaded once per process and cloned out.
static DEFAULT_TOKENIZER: LazyLock<Tokenizer> =
    LazyLock::new(|| init_tokenizer(None, None, false).unwrap());
static MULTILINGUAL_TOKENIZER: LazyLock<Tokenizer> =
    LazyLock::new(|| init_tokenizer(Some(MULTILINGUAL_MODEL.to_string()), None, false).unwrap());

pub fn tokenizer(model: &str) -> Tokenizer {
    match model {
        DEFAULT_MODEL => DEFAULT_TOKENIZER.clone(),
        MULTILINGUAL_MODEL => MULTILINGUAL_TOKENIZER.clone(),
        other => panic!("no cached tokenizer for {other}"),
    }
}

/// Path of `model`'s `tokenizer.json` in the hf-hub cache (downloaded on first use), for
/// loaders such as kitoken that only read local files.
pub fn tokenizer_file(model: &str) -> PathBuf {
    hf_hub::api::sync::Api::new()
        .unwrap()
        .model(model.to_string())
        .get("tokenizer.json")
        .unwrap()
}

pub fn hf_tokenizer(model: &str) -> HFTokenizer {
    HFTokenizer {
        tokenizer: tokenizer(model),
    }
}

/// `SplitterLiteConfig::new_hf` with the tokenizer taken from the cache.
pub fn hf_lite(
    patterns: Vec<Vec<String>>,
    max_tokens: Option<usize>,
    merge_level: Option<usize>,
    parallel: bool,
    model: &str,
) -> SplitterLiteConfig<HFTokenizer> {
    let patterns_len = patterns.len();
    SplitterLiteConfig {
        searchers: searchers(&patterns),
        tokenizer: hf_tokenizer(model),
        max_tokens,
        merge_level: merge_level.map(|level| level.min(patterns_len)),
        parallel: Some(parallel),
        patterns_len,
    }
}

/// Build the split tree over all of `data` through the low-level `Splitter` API.
pub fn split_tree<'a, T: Tokenize + Sync>(
    data: &'a [u8],
    searchers: &'a Vec<PatternSearcher>,
    tokenizer: Option<&'a T>,
    max_len: Option<usize>,
    merge_level: Option<usize>,
) -> SplitNode {
    let config = SplitterConfig {
        data,
        searchers,
        tokenizer,
        max_len,
        merge_level,
        patterns_len: searchers.len(),
    };
    let span = Span {
        start: 0,
        end: data.len(),
    };
    Splitter::new(&config, span, 0, span, None, None, None).split()
}

pub fn texts(splits: &[SplitResultLite]) -> Vec<&str> {
    splits.iter().map(|s| s.split_string.as_str()).collect()
}

/// The splits must tile the input: concatenated back together they are the input, byte for byte.
pub fn assert_tiles(data: &[u8], splits: &[SplitResultLite]) {
    let joined: String = texts(splits).concat();
    assert_eq!(
        joined.as_bytes(),
        data,
        "splits do not reconstruct the input\n{:#?}",
        texts(splits)
    );
}

/// The split tokens must partition the document encoding: same ids in the same order, and
/// no split longer than `max_tokens`.
pub fn assert_token_partition(
    tokenizer: &Tokenizer,
    data: &[u8],
    splits: &[SplitResultLite],
    max_tokens: Option<usize>,
) {
    let document = tokenizer.encode(from_utf8(data).unwrap(), false).unwrap();
    let joined: Vec<u32> = splits
        .iter()
        .flat_map(|s| s.tokens.iter().copied())
        .collect();
    assert_eq!(
        joined,
        document.get_ids(),
        "split tokens differ from the document encoding"
    );
    if let Some(max_tokens) = max_tokens {
        for split in splits {
            assert!(
                split.tokens.len() <= max_tokens,
                "split of {} tokens exceeds max {max_tokens}: {:?}",
                split.tokens.len(),
                split.split_string
            );
        }
    }
}

/// Every split's stored tokens must equal a fresh encoding of its text. Only holds when the
/// splitter never had to cut inside a word, so callers opt in.
pub fn assert_splits_reencode(tokenizer: &Tokenizer, splits: &[SplitResultLite]) {
    for split in splits {
        let encoded = tokenizer
            .encode(split.split_string.as_str(), false)
            .unwrap();
        assert_eq!(
            encoded.get_ids(),
            split.tokens.as_slice(),
            "re-encoding differs for split {:?}",
            split.split_string
        );
    }
}
