use std::fs;
use std::str::from_utf8;

use aho_corasick::Span;
use criterion::{black_box, criterion_main, Criterion};
use rayon::current_num_threads;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use tokenizers::normalizers::bert::BertNormalizer;
use tokenizers::parallelism::{get_parallelism, set_parallelism};
use tokenizers::pre_tokenizers::punctuation::Punctuation;
use tokenizers::pre_tokenizers::sequence::Sequence;
use tokenizers::pre_tokenizers::whitespace::WhitespaceSplit;
use tokenizers::{
    NormalizedString, Normalizer, OffsetReferential, OffsetType, PreTokenizedString, PreTokenizer,
    PreTokenizerWrapper,
};

use fast_text_splitter::config::SplitterLiteConfig;
use fast_text_splitter::encodings::NoneTokenizer;
use fast_text_splitter::hf_tokenizer::init_tokenizer;
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::normalizer::TextNormalizer;
use fast_text_splitter::pattern_search::pattern_searcher::PatternSearcher;
use fast_text_splitter::splitter::Splitter;
use fast_text_splitter::ws_tokenizer::WSTokenizer;

pub fn text_normalize_benchmark(c: &mut Criterion) {
    //let data_path = "tests/error_data/Selena Gomez - Wikipedia.txt";
    let data_path = "tests/test_data/superlinear.txt";

    let data = fs::read_to_string(data_path).unwrap();

    let normalizer = TextNormalizer::new(true, false, None, false);
    c.bench_function("text_normalize", |b| {
        b.iter(|| {
            let normalized = normalizer.normalize(&data);
            let _ = black_box(normalized);
        })
    });
}

pub fn hf_text_normalize_benchmark(c: &mut Criterion) {
    //let data_path = "tests/error_data/Selena Gomez - Wikipedia.txt";
    let data_path = "tests/test_data/superlinear.txt";

    let data = fs::read_to_string(data_path).unwrap();

    let bert_normalizer = BertNormalizer::new(true, false, None, false);

    c.bench_function("hf_text_normalize", |b| {
        b.iter(|| {
            let mut normalized_data = NormalizedString::from(data.clone());
            bert_normalizer.normalize(&mut normalized_data).unwrap();
            let normalized = normalized_data.get().to_string();
            black_box(normalized);
        })
    });
}

pub fn tokenizer_multi_batch_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let content_str = fs::read_to_string(data_path).unwrap();
    let splits = content_str
        .split_terminator("\n\n")
        .map(|p| p.to_string())
        .collect::<Vec<_>>();

    let tokenizer = init_tokenizer(None, Some(content_str.len()), false).unwrap();

    set_parallelism(true);
    println!("Num threads: {}", current_num_threads());
    println!("Parallelism: {}", get_parallelism());

    c.bench_function("tokenizer_multi_batch_benchmark", |b| {
        b.iter(|| {
            let tokens_splits_list: Vec<_> = splits
                .par_iter()
                .map(|split| {
                    tokenizer.encode(split.as_str(), false).unwrap();
                })
                .collect();
            black_box(tokens_splits_list);
        })
    });
}

pub fn hf_tree_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "data/dev/List_of_Game_of_Thrones_characters.txt";
    //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

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

    let hf_tokenizer = HFTokenizer {
        tokenizer: init_tokenizer(None, None, false).unwrap(),
    };

    let max_len = Some(512);
    let data_str = from_utf8(data).unwrap();

    c.bench_function("hf_tree_split_benchmark", |b| {
        b.iter(|| {
            let span = Span {
                start: 0,
                end: data.len(),
            };
            let config =
                fast_text_splitter::splitter::splitter_config::SplitterConfig::<HFTokenizer> {
                    data,
                    searchers: &searchers,
                    max_len,
                    tokenizer: Some(&hf_tokenizer),
                    patterns_len,
                };
            let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
            let tree = splitter.split();
            let merge_level = tree.leaf_level().unwrap_or(0);
            let splits = tree.merge_encoding_result(merge_level, max_len, data_str);
            black_box(splits);
        })
    });
}

pub fn hf_merged_tree_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "data/dev/List_of_Game_of_Thrones_characters.txt";
    //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

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

    let hf_tokenizer = HFTokenizer {
        tokenizer: init_tokenizer(None, None, false).unwrap(),
    };

    let max_len = Some(512);
    let data_str = from_utf8(data).unwrap();

    c.bench_function("hf_merged_tree_split_benchmark", |b| {
        b.iter(|| {
            let span = Span {
                start: 0,
                end: data.len(),
            };
            let config =
                fast_text_splitter::splitter::splitter_config::SplitterConfig::<HFTokenizer> {
                    data,
                    searchers: &searchers,
                    max_len,
                    tokenizer: Some(&hf_tokenizer),
                    patterns_len,
                };
            let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
            let tree = splitter.split();
            let splits = tree.get_results(max_len, data_str);
            black_box(splits);
        })
    });
}

pub fn hf_merged_tree_lite_res_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "data/dev/List_of_Game_of_Thrones_characters.txt";
    //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

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

#[cfg(feature = "default")]
pub fn ws_tree_split_lite_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";

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
            let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
            let tree = splitter.split();
            let splits = tree.get_results_lite(max_len, data);
            black_box(splits);
        })
    });
}

#[cfg(feature = "default")]
pub fn none_tree_split_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";

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

    let max_len = Some(1024);

    c.bench_function("none_tree_split_benchmark", |b| {
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
            let merge_level = tree.leaf_level().unwrap_or(0);
            let splits = tree.merge_splits(merge_level, max_len);
            black_box(splits);
        })
    });
}

pub fn hf_single_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let bytes = fs::read(data_path).unwrap();
    let data = std::str::from_utf8(&bytes).unwrap();
    tokenizers::utils::parallelism::set_parallelism(true);
    let hf_tokenizer = init_tokenizer(None, None, false).unwrap();

    c.bench_function("hf_single_tokenize", |b| {
        b.iter(|| {
            let encoded = hf_tokenizer.encode(data, false).unwrap();
            black_box(encoded);
        })
    });
}

pub fn hf_parallel_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let data = fs::read_to_string(data_path).unwrap();
    let splits = data
        .split("\n\n")
        .map(|p| p.to_string())
        .collect::<Vec<_>>();

    let hf_tokenizer = init_tokenizer(None, None, true).unwrap();

    c.bench_function("hf_parallel_tokenize_benchmark", |b| {
        b.iter(|| {
            let encodings: Vec<_> = splits
                .par_iter()
                .map(|split_str| {
                    hf_tokenizer.encode(split_str.clone(), false).unwrap();
                })
                .collect();
            black_box(encodings);
        })
    });
}

pub fn ws_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let data = fs::read_to_string(data_path).unwrap();
    let splits = data
        .split("\n\n")
        .map(|p| p.to_string())
        .collect::<Vec<_>>();

    let ws_tokenizer = WSTokenizer { ascii: true };

    c.bench_function("ws_tokenize_benchmark", |b| {
        b.iter(|| {
            splits.iter().for_each(|split_str| {
                let ws_tokens = ws_tokenizer.whitespace_punctuation_tokenize(split_str);
                black_box(ws_tokens);
            });
        })
    });
}

pub fn ws_tokenize_whole_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let data = fs::read_to_string(data_path).unwrap();
    let data = data.as_str();
    let ws_tokenizer = WSTokenizer { ascii: true };

    c.bench_function("ws_tokenize_whole_benchmark", |b| {
        b.iter(|| {
            let ws_tokens = ws_tokenizer.whitespace_punctuation_tokenize(data);
            black_box(ws_tokens);
        })
    });
}

pub fn hf_pre_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    let data = fs::read_to_string(data_path).unwrap();
    let data = data.as_str();

    let splits: Vec<_> = data.split("\n\n").map(|p| p).collect();

    let pretokenizers = vec![
        PreTokenizerWrapper::WhitespaceSplit(WhitespaceSplit),
        PreTokenizerWrapper::Punctuation(Punctuation::default()),
    ];
    let pretok = Sequence::new(pretokenizers);

    c.bench_function("hf_pre_tokenize_benchmark", |b| {
        b.iter(|| {
            splits.par_iter().for_each(|split_str| {
                let mut pretokenized: PreTokenizedString = (*split_str).into();
                pretok.pre_tokenize(&mut pretokenized).unwrap();
                let offset = pretokenized
                    .get_splits(OffsetReferential::Original, OffsetType::Byte)
                    .into_iter()
                    .map(|(s, o, _)| (s, o))
                    .collect::<Vec<_>>();
                black_box(offset);
            });
        })
    });
}

pub fn benches() {
    let mut criterion: Criterion<_> = Criterion::default()
        .sample_size(40)
        .measurement_time(std::time::Duration::from_secs(10))
        .configure_from_args();
    //hf_text_normalize_benchmark(&mut criterion);
    //text_normalize_benchmark(&mut criterion);
    //#[cfg(feature = "default")]
    //hf_pre_tokenize_benchmark(&mut criterion);
    //ws_tokenize_benchmark(&mut criterion);
    //ws_tokenize_whole_benchmark(&mut criterion);
    //hf_parallel_tokenize_benchmark(&mut criterion);

    //tokenizer_multi_batch_benchmark(&mut criterion);
    //hf_tree_split_benchmark(&mut criterion);
    //hf_merged_tree_split_benchmark(&mut criterion);
    hf_merged_tree_lite_res_benchmark(&mut criterion);
    ws_tree_split_lite_benchmark(&mut criterion);
    //none_tree_split_benchmark(&mut criterion);

    //hf_single_tokenize_benchmark(&mut criterion);
}

criterion_main!(benches);
