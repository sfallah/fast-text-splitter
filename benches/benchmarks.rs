use std::fs;

use criterion::{black_box, criterion_main, Criterion};

use fast_text_splitter::config::SplitterLiteConfig;

pub fn hf_merged_tree_lite_res_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";

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
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();
    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];

    let splitter_config = SplitterLiteConfig::new_ws(patterns, 384, 0, true, true);
    c.bench_function("ws_tree_split_benchmark", |b| {
        b.iter(|| {
            let splits = splitter_config.ws_splits(data);
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

    let splitter_config = SplitterLiteConfig::new_none(patterns, 512, 0, false);

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
    none_tree_split_lite_benchmark(&mut criterion);
}

criterion_main!(benches);
