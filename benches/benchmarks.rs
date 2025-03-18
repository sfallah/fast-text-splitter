use std::fs;

use criterion::{black_box, criterion_main, Criterion};

use fast_text_splitter::config::SplitterLiteConfig;
use rayon::prelude::*;

pub fn hf_merged_tree_lite_res_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";
    //let data_path = "tests/llm_papers_txt/2108.07258v3.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();

    let patterns = vec![
        vec!["<SENT>".to_string()],
        vec!["\n\n".to_string()],
        vec!["\n".to_string()]
    ];

    let splitter_config = SplitterLiteConfig::new_hf(patterns, Some(512), None, true, None);

    c.bench_function("hf_merged_tree_lite_splits_benchmark", |b| {
        b.iter(|| {
            let splits = splitter_config.hf_splits(data);
            black_box(splits);
        })
    });
}

pub fn tokenize_sentences_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();

    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["<SENT>".to_string()],
        vec!["\n".to_string()],
        vec![
            ". ".to_string(),
            "! ".to_string(),
            "? ".to_string(),
        ],
    ];

    let splitter_config = SplitterLiteConfig::new_hf(patterns, Some(512), None, true, None);

    c.bench_function("hf_merged_tree_lite_splits_benchmark", |b| {
        b.iter(|| {
            let splits = splitter_config.hf_splits(data);
            black_box(splits);
        })
    });
}

pub fn hf_merged_tree_lite_sub_res_benchmark(c: &mut Criterion) {
    //let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    let data_path = "tests/test_data/bert_paper_complete.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();

    let patterns = vec![
        vec!["\n\n".to_string()],
        vec![
            ". ".to_string(),
            "! ".to_string(),
            "? ".to_string(),
        ],
        vec!["\n".to_string()],
    ];

    let splitter_config =
        SplitterLiteConfig::new_hf(patterns.clone(), Some(512), None, false, None);
    let sub_splitter_config = SplitterLiteConfig::new_hf(
        patterns.clone(),
        Some(512),
        Some(patterns.len()),
        false,
        None,
    );

    c.bench_function("hf_merged_tree_lite_sub_res_benchmark", |b| {
        b.iter(|| {
            let sub_splits: Vec<_> = splitter_config
                .hf_splits(data)
                .par_iter()
                .flat_map(|split| sub_splitter_config.hf_splits(split.split_string.as_bytes()))
                .collect();
            black_box(sub_splits);
        })
    });
}

pub fn ws_tree_split_lite_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";
    //let data_path = "tests/llm_papers_txt/2108.07258v3.txt";




    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();
    let patterns = vec![
        vec!["<SENT>".to_string()],
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
    ];

    let splitter_config = SplitterLiteConfig::new_ws(patterns, Some(384), None, true, true);
    c.bench_function("ws_tree_split_benchmark", |b| {
        b.iter(|| {
            let splits = splitter_config.ws_splits(data);
            black_box(splits);
        })
    });
}

pub fn none_tree_split_lite_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    // data_path = "tests/test_data/United_States.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();
    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];

    let splitter_config = SplitterLiteConfig::new_none(patterns, Some(512), None, false);

    c.bench_function("none_tree_split_lite_benchmark", |b| {
        b.iter(|| {
            let splits = splitter_config.len_splits(data);
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
    //none_tree_split_lite_benchmark(&mut criterion);
    //hf_merged_tree_lite_sub_res_benchmark(&mut criterion);
}

criterion_main!(benches);
