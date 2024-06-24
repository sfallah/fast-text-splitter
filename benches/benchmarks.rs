use aho_corasick::Span;
use criterion::{black_box, criterion_main, Criterion};
use fast_text_splitter::config::SplitterLiteConfig;
use fast_text_splitter::encodings::NoneTokenizer;
use fast_text_splitter::pattern_search::pattern_searcher::PatternSearcher;
use fast_text_splitter::splitter::Splitter;
use fast_text_splitter::ws_tokenizer::WSTokenizer;
use std::fs;

pub fn hf_merged_tree_lite_res_benchmark(c: &mut Criterion) {
    //let data_path = "tests/test_data/superlinear.txt";
    let data_path = "tests/test_data/United_States.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();

    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];

    let splitter_config = SplitterLiteConfig::new_hf(patterns, 512, 0, true, None);

    c.bench_function("hf_merged_tree_lite_splits_benchmark", |b| {
        b.iter(|| {
            let splits = splitter_config.hf_splits(data);
            black_box(splits);
        })
    });
}

pub fn ws_tree_split_lite_benchmark(c: &mut Criterion) {
    //let data_path = "tests/test_data/superlinear.txt";
    let data_path = "tests/test_data/United_States.txt";


    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();
    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];
    let patterns_len = patterns.len();
    let searchers: Vec<_> = patterns
        .iter()
        .map(|p| PatternSearcher::new(p.clone()))
        .collect();

    let ws_tokenizer = WSTokenizer { ascii: true };

    let max_len = Some(384);

    c.bench_function("ws_tree_split_benchmark", |b| {
        b.iter(|| {
            let span = Span {
                start: 0,
                end: data.len(),
            };
            let config =
                fast_text_splitter::splitter::splitter_config::SplitterConfig::<WSTokenizer> {
                    data,
                    searchers: &searchers,
                    max_len,
                    tokenizer: Some(&ws_tokenizer),
                    patterns_len,
                };
            let splitter = Splitter::new(&config, span, 0, span, Some(false), None, None);
            let tree = splitter.split();
            let splits = tree.get_results_lite(max_len, data);
            black_box(splits);
        })
    });
}

pub fn none_tree_split_lite_benchmark(c: &mut Criterion) {
    //let data_path = "tests/test_data/superlinear.txt";
    let data_path = "tests/test_data/United_States.txt";


    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();
    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];
    let patterns_len = patterns.len();
    let searchers: Vec<_> = patterns
        .iter()
        .map(|p| PatternSearcher::new(p.clone()))
        .collect();

    let max_len = Some(156);

    c.bench_function("none_tree_split_lite_benchmark", |b| {
        b.iter(|| {
            let span = Span {
                start: 0,
                end: data.len(),
            };
            let config =
                fast_text_splitter::splitter::splitter_config::SplitterConfig::<NoneTokenizer> {
                    data,
                    searchers: &searchers,
                    max_len,
                    tokenizer: None,
                    patterns_len,
                };
            let splitter = Splitter::new(&config, span, 0, span, Some(false), None, None);
            let tree = splitter.split();
            let splits = tree.get_results_lite(max_len, data);
            black_box(splits);
        })
    });
}

pub fn benches() {
    let mut criterion: Criterion<_> = Criterion::default()
        .sample_size(40)
        .measurement_time(std::time::Duration::from_secs(10))
        .configure_from_args();

    hf_merged_tree_lite_res_benchmark(&mut criterion);
    ws_tree_split_lite_benchmark(&mut criterion);
    none_tree_split_lite_benchmark(&mut criterion);
}

criterion_main!(benches);
