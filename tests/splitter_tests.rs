use aho_corasick::Span;
use fast_text_splitter::splitter::split;
use std::fs;
use std::str::from_utf8;
use fast_text_splitter::pattern_search::PatternSearcher;

#[test]
fn single_level_split() {
    let data = "Hello, you all.\n How are you.\n".as_bytes();
    let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
    let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
    let span = Span {
        start: 0,
        end: data.len(),
    };
    let tree = split(data, span, &patterns, 0, span, &searchers);
    println!("{:?}", tree);
    println!("{}", tree.to_string(true));
    let reconsted = tree.reconstruct();
    assert_eq!(from_utf8(data).unwrap(), reconsted);

    let merged_0 = tree.merge(0, false);
    assert_eq!(merged_0.len(), 1);
    assert_eq!(data.len(), merged_0[0].len());

    println!("#### Merged Level 0 Checks ####");
    for span in merged_0.iter() {
        println!("\t {:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }

    println!("#### Merged Level 1 Checks ####");
    let merged_1 = tree.merge(1, false);
    assert_eq!(merged_1.len(), 2);
    let merged_1_len_sum: usize = merged_1.iter().map(|span| span.len()).sum();
    assert_eq!(data.len(), merged_1_len_sum);

    for span in merged_1.iter() {
        println!("\t {:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }

    println!("#### Merged Level 2 Checks ####");
    let merged_2 = tree.merge(2, false);
    assert_eq!(merged_2.len(), 2);
    for span in merged_2.iter() {
        println!("\t {:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
}

#[test]
fn multi_level_split() {
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
    let tree = split(data, span, &patterns, 0, span, &searchers);

    println!("{}", tree.to_string(true));
    assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());

    println!("#### Merged Level 0 Checks ####");
    let merged_level_0 = tree.merge(0, false);
    assert_eq!(merged_level_0.len(), 8);
    let splits_len_sum: usize = merged_level_0.iter().map(|span| span.len()).sum();
    assert_eq!(data.len(), splits_len_sum);

    for span in merged_level_0.iter() {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }

    let merged_no_empty_level_0: Vec<_> = tree.merge(0, true);
    assert_eq!(merged_no_empty_level_0.len(), 3);

    for span in merged_no_empty_level_0.iter() {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }

    println!("\n #### Merged Level 1 Checks ####");
    let merged_level_1 = tree.merge(1, false);
    assert_eq!(merged_level_1.len(), 8);
    let merged_no_empty_level_1: Vec<_> = tree.merge(1, true);
    assert_eq!(merged_no_empty_level_1.len(), 7);

    for span in merged_no_empty_level_1.iter() {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }

    println!("\n #### Merged Level 2 Checks ####");
    let merged_level_2 = tree.merge(2, false);
    for span in merged_level_2.iter() {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
    assert_eq!(merged_level_2.len(), 8);
    let merged_no_empty_level_2: Vec<_> = tree.merge(2, true);
    assert_eq!(merged_no_empty_level_2.len(), 8);
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
    let tree = split(data, span, &patterns, 0, span, &searchers);
    println!("{}", tree.to_string(true));
    assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());

    let merged = tree.merge(1, false);
    for span in merged {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
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
    let tree = split(data, span, &patterns, 0, span, &searchers);
    println!("{}", tree.to_string(true));
    let reconsted = tree.reconstruct();
    assert_eq!(from_utf8(&data).unwrap(), reconsted);

    let merged_0 = tree.merge(0, false);
    let splits_len_sum: usize = merged_0.iter().map(|span| span.len()).sum();
    assert_eq!(data.len(), splits_len_sum);
    let merged_0_filtered: Vec<_> = tree.merge(0, true);

    let merged_1 = tree.merge(1, false);
    let merged_1_filtered: Vec<_> = tree.merge(1, true);
    let merged_2 = tree.merge(2, false);
    let merged_2_filtered: Vec<_> = tree.merge(2, true);
    println!("Merged 0: {:?}", merged_0.len());
    println!("Merged 0 Filtered: {:?}", merged_0_filtered.len());
    println!("Merged 1: {:?}", merged_1.len());
    println!("Merged 1 Filtered: {:?}", merged_1_filtered.len());
    println!("Merged 2: {:?}", merged_2.len());
    println!("Merged 2 Filtered: {:?}", merged_2_filtered.len());

    let merged_spans_filtered: Vec<_> = merged_1.iter().filter(|span| !span.is_empty()).collect();

    println!("Merged Spans: {:?}", merged_spans_filtered.len());
    for span in merged_spans_filtered {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
}
