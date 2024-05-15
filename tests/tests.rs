use std::fs;

use aho_corasick::{AhoCorasick, MatchKind, Span};
use tokenizers::{PaddingStrategy, Tokenizer};
use fast_text_splitter::config::SplitterConfig;
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::{text_split, text_split_parallel};
use fast_text_splitter::ws_tokenizer::{whitespace_indices, word_spans, ws_spans, WSTokenizer};

#[cfg(feature = "tokenizers")]

#[cfg(feature = "tokenizers")]
#[test]
fn hf_parallel_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you foo and poo? \n\n I am fine. Nice to meet you all insecure!";

    let conf: SplitterConfig<HFTokenizer> =
        SplitterConfig::new_hf_config(None, None, 8, 2, None, true);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.splits);
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn ws_parallel_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you ? \n\n I am fine. Nice to meet you all insecure!";

    let conf: SplitterConfig<WSTokenizer> = SplitterConfig::default();

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn no_matches_ws_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf: SplitterConfig<WSTokenizer> = SplitterConfig::default();

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    let conf: SplitterConfig<WSTokenizer> = SplitterConfig::new_ws_config(None, 8, 2, None, true);
    let splits = text_split_parallel(&conf, data);
    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn no_matches_hf_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf: SplitterConfig<HFTokenizer> = SplitterConfig::default();

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    let conf: SplitterConfig<HFTokenizer> = SplitterConfig::new_hf_config(None, None, 8, 2, None, true);
    let splits = text_split_parallel(&conf, data);
    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}
#[test]
fn split_words_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";

    let bytes = fs::read(data_path)?;
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf: SplitterConfig<WSTokenizer> = SplitterConfig::default();

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.splits);
        println!("{:?}", split.split_strings);
    }
    Ok(())
}

#[test]
fn split_tokenizer_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";

    let bytes = fs::read(data_path)?;
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf = SplitterConfig::new_hf_config(None, None, 68, 2, Some(1), true);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.splits);
        println!("{:?}", split.split_strings);
    }
    Ok(())
}

#[test]
fn ws_spans_test() {
    let data = " Mary had\ta little  \n\t lamb".to_string();
    println!("data len:{}", data.len());

    let indexes: Vec<_> = whitespace_indices(&data);
    println!("indexes: {:?}", indexes);

    let ws_indices = whitespace_indices(&data);
    println!("ws indices: {:?}", ws_indices);
    assert_eq!(indexes, ws_indices);

    let spans = ws_spans(indexes);

    println!("spans: {:?}", spans);

    let word_spans = word_spans(spans, data.len());
    println!("word spans: {:?}", word_spans);

    let mut words_ws = vec![];
    for span in word_spans.iter() {
        println!("{:?}", span);
        words_ws.push(&data[span.start..span.end]);
        println!("{:?}", &data[span.start..span.end]);
    }

    let words = data.split_whitespace();

    for (idx, word) in words.enumerate() {
        println!("{}: {}", idx, word);
        let word_ws = words_ws[idx];
        assert!(word_ws.contains(word));
    }
}

#[test]
fn memchr_whitespace_test() {
    let data = " Mary had\ta little  \n\t lamb".to_string();
    println!("data len:{}", data.len());
    let indexes: Vec<_> = whitespace_indices(&data);
    let mut word_indexes: Vec<usize> = (0..data.len()).collect();
    word_indexes.retain(|&x| !indexes.contains(&x));

    println!("word indexes: {:?}", word_indexes);

    for idx in indexes.iter() {
        let chr = &data[*idx..idx + 1];
        let chr_pres = match chr {
            " " => "space",
            "\t" => "tab",
            "\n" => "newline",
            _ => "unknown",
        };

        println!("idex: {:?}, char: {}", idx, chr_pres);
    }

    let spans: Vec<Span> = vec![];

    for span in spans {
        println!("{:?}", span);
        println!("{:?}", &data[span.start..span.end])
    }
}

#[test]
fn aho_corasick_span_tests() {
    let span = Span { start: 0, end: 1 };
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 1);

    assert_eq!(span.len(), 1);
}

#[test]
fn aho_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you ?\n\n I am fine. Nice to meet you all insecure!";

    let mut tokenizer = Tokenizer::from_pretrained("sentence-transformers/all-MiniLM-L6-v2", None)?;
    tokenizer.get_padding_mut().unwrap().strategy = PaddingStrategy::BatchLongest;
    let tokenizer_truncation = tokenizer.get_truncation_mut().unwrap();
    tokenizer_truncation.strategy = tokenizers::TruncationStrategy::LongestFirst;
    tokenizer_truncation.max_length = 512;

    let patterns = &["\n\n", "\n"];
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)
        .unwrap();

    // measure time
    let start = std::time::Instant::now();
    let encoded = tokenizer.encode(data, false)?;
    let duration = start.elapsed();
    println!("Encoding time: {:?}", duration);

    let matches = ac.find_iter(data).collect::<Vec<_>>();

    let conf: SplitterConfig<WSTokenizer> = SplitterConfig::default();

    let splits_level1 = text_split(
        &encoded.get_offsets(),
        &encoded.get_word_ids(),
        Span {
            start: 0,
            end: encoded.len(),
        },
        &matches,
        Span {
            start: 0,
            end: matches.len(),
        },
        0,
        &conf,
    );

    println!("## Splits {:?}", splits_level1);

    Ok(())
}

#[test]
fn tokenizer_multi_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";
    let content_str = fs::read_to_string(data_path)?;
    let splits: Vec<&str> = content_str.split_terminator("\n\n").collect();

    for split in splits.iter() {
        println!("{:?}", split.split_whitespace().count());
    }

    println!("Splits: {:?}", splits.len());

    let model = "sentence-transformers/all-MiniLM-L6-v2".to_string();
    let mut tokenizer = Tokenizer::from_pretrained(model, None)?;
    tokenizer.get_padding_mut().unwrap().strategy = PaddingStrategy::BatchLongest;

    let tokenizer_truncation = tokenizer.get_truncation_mut().unwrap();
    tokenizer_truncation.strategy = tokenizers::TruncationStrategy::LongestFirst;
    tokenizer_truncation.max_length = 512;

    let encoded = tokenizer.encode_batch(splits, true)?;

    for enc in encoded.iter() {
        println!("{:?}", enc.len());
        println!("{:?}", &enc.get_ids());
    }

    Ok(())
}
