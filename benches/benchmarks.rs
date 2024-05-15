use std::fs;

use criterion::{black_box, criterion_main, Criterion};

use fast_text_splitter::config::SplitterConfig;
#[cfg(feature = "tokenizers")]
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::text_split_parallel;

use fast_text_splitter::ws_tokenizer::WSTokenizer;

#[cfg(feature = "tokenizers")]
pub fn tokenizer_single_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let bytes = fs::read(data_path).unwrap();
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf: SplitterConfig<HFTokenizer> = SplitterConfig::default();

    c.bench_function("tokenizer_single_split", |b| {
        b.iter(|| {
            let splits =text_split_parallel(&conf, data);
            black_box(splits);
        })
    });
}

#[cfg(feature = "default")]
pub fn words_single_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let bytes = fs::read(data_path).unwrap();
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf: SplitterConfig<WSTokenizer> = SplitterConfig::default();

    c.bench_function("words_single_split", |b| {
        b.iter(|| {
            let splits = text_split_parallel(&conf, data);
            black_box(splits);
        })
    });
}

pub fn benches() {
    let mut criterion: Criterion<_> = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(10))
        .configure_from_args();
    #[cfg(feature = "default")]
    words_single_split_benchmark(&mut criterion);
    #[cfg(feature = "tokenizers")]
    tokenizer_single_split_benchmark(&mut criterion);
}

criterion_main!(benches);
