use std::str::from_utf8;
use std::{fs, io};

use aho_corasick::Span;

use fast_text_splitter::pattern_search::PatternSearcher;
use fast_text_splitter::splitter::{chunk_spans, split};

use fast_text_splitter::common::span;
use rand::seq::SliceRandom;
use rand::thread_rng;

fn list_text_files(dir: &str) -> io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            files.push(String::from(path.to_str().unwrap()));
        }
    }

    Ok(files)
}

#[test]
fn single_level_split() {
    let data = "Hello, you all.\n How are you.\n".as_bytes();
    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
    let span = Span {
        start: 0,
        end: data.len(),
    };
    let tree = split(data, span, &patterns, 0, span, &searchers, None,None);
    println!("{:?}", tree);
    println!("{}", tree.to_string(true));
    let reconsted = tree.reconstruct();
    assert_eq!(from_utf8(data).unwrap(), reconsted);
}

#[test]
fn multi_level_split() {
    let _data_raw = "\n\n\
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
        Outperform everyone else.\n\n"
        .as_bytes();

    let data = _data_raw;

    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    let span = Span {
        start: 0,
        end: data.len(),
    };
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
    let tree = split(data, span, &patterns, 0, span, &searchers, None,None);

    println!("{}", tree.to_string(true));
    assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());
}

#[test]
fn max_len_splits() {
    let _data_raw = "\n\n\
        \n\n\
        \n\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\n\
        Another heuristic for.\n\
        \n\n\
        \n\n\
        \n\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n\n"
        .as_bytes();

    let data = _data_raw;

    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    let span = Span {
        start: 0,
        end: data.len(),
    };
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
    let tree = split(data, span, &patterns, 0, span, &searchers, None,None);

    println!("{}", tree.to_string(true));
    assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());

    /*
    let merged_level_0 = tree.merge(0, false);
    for span in merged_level_0.iter() {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
     */

    let tree_leaves = tree.all_leaves();
    for span in tree_leaves.iter() {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
    let total_len: usize = tree_leaves.iter().map(|span| span.len()).sum();
    let leaves_concatenated: String = tree_leaves
        .iter()
        .map(|span| from_utf8(&data[span.start..span.end]).unwrap())
        .collect();
    println!("{:?}", leaves_concatenated);
    assert_eq!(data.len(), total_len);
}

#[test]
fn first_pattern_no_match_split() {
    let data = "\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\
        Another heuristic for.\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n"
        .as_bytes();

    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    //let patterns = vec!["\n\n".to_string()];
    //let patterns = vec!["\n\n".to_string(), "\n".to_string()];
    let span = Span {
        start: 0,
        end: data.len(),
    };
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
    let tree = split(data, span, &patterns, 0, span, &searchers, None,None);
    println!("{}", tree.to_string(true));
    assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());
}

#[test]
fn pattern_split_superlinear_test() {
    let data_path = "tests/test_data/superlinear.txt";

    let binding = fs::read_to_string(data_path).unwrap();
    let data = binding.as_bytes();

    let patterns = vec![vec!["\n\n"], vec!["\n"], vec!["."]];

    let span = Span {
        start: 0,
        end: data.len(),
    };

    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
    let tree = split(data, span, &patterns, 0, span, &searchers, None,None);
    println!("{}", tree.to_string(true));
    let reconsted = tree.reconstruct();
    assert_eq!(from_utf8(&data).unwrap(), reconsted);
}

#[test]
fn pattern_split_superlinear_print() {
    //let data_path = "tests/test_data/superlinear.txt";
    let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();

    let (data_len, splits) = split_file(data_path, &patterns, &searchers, 1024, true);

    let total_len: usize = splits.iter().map(|span| span.len()).sum();
    assert_eq!(data_len, total_len);
    println!("Total Leaves: {:?}", splits.len());
    println!("Total Length: {:?}", total_len);
}

fn split_file<'a>(
    data_path: &str,
    patterns: &Vec<Vec<&str>>,
    searchers: &Vec<PatternSearcher>,
    max_len: usize,
    _print: bool,
) -> (usize, Vec<Span>) {
    let binding = fs::read_to_string(data_path).unwrap();
    let data = binding.as_bytes();

    let span = Span {
        start: 0,
        end: data.len(),
    };
    let tree = split(data, span, &patterns, 0, span, &searchers, Some(max_len), None);
    let leaf_level = tree.leaf_level().unwrap_or(0);
    println!("Leaf Level: {:?}", leaf_level);
    let splits = tree.merge_splits(leaf_level, Some(max_len));
    for span in splits.iter() {
        assert!(span.len() <= max_len, "Failed File: {} \n Span {:?} \n {}", span.len(), span, from_utf8(&data[span.start..span.end]).unwrap());
    }

    if _print {
        for span in splits.iter() {
            println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
        }
    }
    (data.len(), splits)
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_test() -> tokenizers::Result<()> {
    let mut rng = thread_rng();

    tokenizers::utils::parallelism::set_parallelism(true);
    let files = list_text_files("data/dev/")?;

    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();

    files.choose_multiple(&mut rng, 400).for_each(|file| {
        let (data_len, splits) = split_file(file, &patterns, &searchers, 512, false);
        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        assert_eq!(data_len, total_len, "Failed File: {}", file);
        println!("File: {} \n Total Length: {}", file, total_len);
        println!("Number of Splits: {:?}", splits.len());
    });

    Ok(())
}

#[test]
fn chunk_spans_test() {
    let single_span = vec![span(0, 10)];
    let chunked = chunk_spans(&single_span, 10);
    assert_eq!(chunked.len(), 1);
    assert_eq!(chunked[0].len(), 10);

    let even_spans = vec![span(0, 10), span(10, 16), span(16, 36), span(36, 45)];
    let chunked = chunk_spans(&even_spans, 20);
    assert_eq!(chunked.len(), 3);
    assert_eq!(chunked[0], span(0, 16));
    assert_eq!(chunked[0].len(), 16);
    assert_eq!(chunked[1], span(16, 36));
    assert_eq!(chunked[1].len(), 20);
    assert_eq!(chunked[2], span(36, 45));
    assert_eq!(chunked[2].len(), 9);

    let odd_spans = vec![span(0, 10), span(10, 16), span(16, 30), span(30, 35), span(35, 40)];
    let chunked = chunk_spans(&odd_spans, 20);
    assert_eq!(chunked.len(), 3);
}
