use std::fs;

use criterion::{black_box, Criterion, criterion_main};
use tokenizers::{NormalizedString, Normalizer};
use tokenizers::normalizers::bert::BertNormalizer;

use fast_text_splitter::ac_matches::init_aho_corasick;
use fast_text_splitter::config::{ConfigParams, SplitterConfig};
#[cfg(feature = "tokenizers")]
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::normalizer::TextNormalizer;
use fast_text_splitter::text_split_parallel;
use fast_text_splitter::ws_tokenizer::WSTokenizer;

#[cfg(feature = "tokenizers")]
pub fn tokenizer_single_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "data/train/Commonwealth_of_Nations.txt";
    //let data_path = "data/dev/Electroencephalography - Wikipedia.txt";
    let bytes = fs::read(data_path).unwrap();
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(data.len())
        .merge_level(0)
        .max_depth(3)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    c.bench_function("tokenizer_single_split", |b| {
        b.iter(|| {
            let splits = text_split_parallel(&conf, data, Some(false));
            black_box(splits);
        })
    });
}

pub fn text_normalize_benchmark(c: &mut Criterion) {
    //let data_path = "tests/error_data/Selena Gomez - Wikipedia.txt";
    let data_path = "tests/test_data/superlinear.txt";

    let data = fs::read_to_string(data_path).unwrap();

    let normalizer = TextNormalizer::new(true, false, None, false);
    c.bench_function("text_normalize", |b| {
        b.iter(|| {
            let normalized = normalizer.normalize(&data);
            black_box(normalized);
        })
    });
}

pub fn hf_text_normalize_benchmark(c: &mut Criterion) {
    //let data_path = "tests/error_data/Selena Gomez - Wikipedia.txt";
    let data_path = "tests/test_data/superlinear.txt";

    let data = fs::read_to_string(data_path).unwrap();

    let mut bert_normalizer = BertNormalizer::new(true, false, None, false);

    c.bench_function("hf_text_normalize", |b| {
        b.iter(|| {
            let mut normalized_data = NormalizedString::from(data.clone());
            bert_normalizer.normalize(&mut normalized_data).unwrap();
            let normalized = normalized_data.get().to_string();
            black_box(normalized);
        })
    });
}

pub fn aho_corasick_init_benchmark(c: &mut Criterion) {
    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];

    c.bench_function("aho_corasick_init", |b| {
        b.iter(|| {
            patterns.iter().map(|p| {
                let ac = init_aho_corasick(p).unwrap();
                black_box(ac);
            })
        })
    });
}

#[cfg(feature = "default")]
pub fn words_single_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "data/train/Commonwealth_of_Nations.txt";
    //let data_path = "data/dev/Electroencephalography - Wikipedia.txt";
    let bytes = fs::read(data_path).unwrap();
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(data.len())
        .merge_level(0)
        .max_depth(3)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    c.bench_function("words_single_split", |b| {
        b.iter(|| {
            let splits = text_split_parallel(&conf, data, Some(false));
            black_box(splits);
        })
    });
}

pub fn benches() {
    let mut criterion: Criterion<_> = Criterion::default()
        .sample_size(40)
        .measurement_time(std::time::Duration::from_secs(10))
        .configure_from_args();
    //aho_corasick_init_benchmark(&mut criterion);
    //hf_text_normalize_benchmark(&mut criterion);
    //text_normalize_benchmark(&mut criterion);
    #[cfg(feature = "default")]
    words_single_split_benchmark(&mut criterion);
    #[cfg(feature = "tokenizers")]
    tokenizer_single_split_benchmark(&mut criterion);
}

criterion_main!(benches);
