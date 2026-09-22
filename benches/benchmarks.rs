use criterion::{criterion_main, Criterion};
use fast_text_splitter::config::SplitterLiteConfig;
use fast_text_splitter::encodings::Tokenize;
use fast_text_splitter::hf_tokenizer::init_tokenizer;
use fast_text_splitter::pattern_search::pattern_searcher::icu_sentence_tokenize;
use kitoken::{Definition, Kitoken, Processing, TokenId};
use punkt::{sentence_tokenize_lang, SentenceToken};
use rayon::prelude::*;
use std::fs;
use std::hint::black_box;

pub fn hf_merged_tree_lite_res_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";
    //let data_path = "tests/llm_papers_txt/2108.07258v3.txt";

    let binding = fs::read(data_path).unwrap();
    let data = binding.as_slice();

    let patterns = vec![vec!["<ICU_SENT>".to_string()]];

    let model_id = "openai-community/gpt2";
    let model_id = "sentence-transformers/all-MiniLM-L6-v2";

    let splitter_config = SplitterLiteConfig::new_hf_with_normalizer_opt(
        patterns,
        Some(512),
        None,
        true,
        Some(true),
        Some(model_id.to_string()),
    );

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
        vec!["<SENT>".to_string()],
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![". ".to_string(), "! ".to_string(), "? ".to_string()],
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
        vec![". ".to_string(), "! ".to_string(), "? ".to_string()],
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
pub fn ws_tokenize_words_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();

    let word_tokenizer = fast_text_splitter::ws_tokenizer::WSTokenizer { ascii: true };

    c.bench_function("ws_tokenize_words_benchmark", |b| {
        b.iter(|| {
            let words = word_tokenizer.encode(data.as_str()).unwrap();
            black_box(words);
        })
    });
}

pub fn sent_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();

    let model_id = "openai-community/gpt2";
    //let model_id = "sentence-transformers/all-MiniLM-L6-v2";

    let tokenizer = init_tokenizer(Some(model_id.to_string()), Some(512), true).unwrap();

    c.bench_function("sent_tokenize_benchmark", |b| {
        b.iter(|| {
            let sentences = sentence_tokenize_lang(&data, Some("english")).unwrap();
            let encodings = black_box(sentences)
                .par_iter()
                .map(|s| tokenizer.encode_fast(s.text.as_str(), false).unwrap())
                .collect::<Vec<_>>();
            black_box(encodings);
        })
    });
}

pub fn sent_tik_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();
    let bpe_tokenizer = tiktoken_rs::r50k_base().unwrap();

    c.bench_function("sent_tik_tokenize_benchmark", |b| {
        b.iter(|| {
            let sentences = sentence_tokenize_lang(&data, Some("english")).unwrap();
            let encodings = black_box(sentences)
                .par_iter()
                .map(|s| bpe_tokenizer.encode_with_special_tokens(s.text.as_str()))
                .collect::<Vec<_>>();
            black_box(encodings);
        })
    });
}

pub fn sent_kitoken_tokenize_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();
    let model_file = "models/openai-community/gpt2/tokenizer.json";
    //let model_file = "models/openai-community/sentence-transformers/all-MiniLM-L6-v2/tokenizer.json";

    let mut definition = Definition::from_tokenizers_file(model_file).unwrap();
    let processing = definition
        .config
        .processing
        .iter()
        .flat_map(|m| match m {
            Processing::Pad { .. } => None,
            Processing::Truncate {
                length: _,
                stride,
                direction,
            } => Some(Processing::Truncate {
                length: 512,
                stride: *stride,
                direction: *direction,
            }),
            _ => Some(m.clone()),
        })
        .collect::<Vec<_>>();
    definition.config.processing = processing;
    let ki_tokenizer = Kitoken::from_definition(definition).unwrap();

    c.bench_function("sent_kitoken_tokenize_benchmark", |b| {
        b.iter(|| {
            let sentences = sentence_tokenize_lang(&data, Some("english")).unwrap();
            let encodings = black_box(sentences)
                .par_iter()
                .map(|s| ki_tokenizer.encode(s.text.as_str(), false).unwrap())
                .collect::<Vec<_>>();
            black_box(encodings);
        })
    });
}

fn simple_splits(
    sentences: &Vec<SentenceToken>,
    ki_encodings: &Vec<Vec<TokenId>>,
) -> Vec<(String, usize)> {
    let max_len = 400;
    let no_sentences = sentences.len();

    let splits = ki_encodings.iter().zip(sentences.iter()).enumerate().fold(
        (Vec::new(), Vec::new(), 0),
        |mut acc, (idx, (encoding, sentence))| {
            if acc.2 + encoding.len() > max_len {
                if !acc.1.is_empty() {
                    acc.0.push((acc.1.join(" "), acc.2));
                    acc.1.clear();
                    acc.2 = 0;
                }
            }
            acc.1.push(sentence.text.clone());
            acc.2 += encoding.len();
            if idx == no_sentences - 1 {
                acc.0.push((acc.1.join(" "), acc.2 + encoding.len()));
            }
            acc
        },
    );

    let splits = splits.0;
    splits
}

pub fn sent_kitoken_simple_splits_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();
    let model_file = "models/openai-community/gpt2/tokenizer.json";
    let model_file =
        "models/openai-community/sentence-transformers/all-MiniLM-L6-v2/tokenizer.json";

    let mut definition = Definition::from_tokenizers_file(model_file).unwrap();
    let processing = definition
        .config
        .processing
        .iter()
        .flat_map(|m| match m {
            Processing::Pad { .. } => None,
            Processing::Truncate {
                length: _,
                stride,
                direction,
            } => Some(Processing::Truncate {
                length: 512,
                stride: *stride,
                direction: *direction,
            }),
            _ => Some(m.clone()),
        })
        .collect::<Vec<_>>();
    definition.config.processing = processing;
    let ki_tokenizer = Kitoken::from_definition(definition).unwrap();

    c.bench_function("sent_kitoken_simple_splits_benchmark", |b| {
        b.iter(|| {
            let sentences = sentence_tokenize_lang(&data, Some("english")).unwrap();
            let encodings = sentences
                .par_iter()
                .map(|s| ki_tokenizer.encode(s.text.as_str(), false).unwrap())
                .collect::<Vec<_>>();
            let splits = simple_splits(&sentences, &encodings);
            black_box(splits);
        })
    });
}

pub fn sent_tokenize_benchmark_paral_batch(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();
    let patterns = vec![vec!["\n\n".to_string()], vec!["<ICU_SENT>".to_string()]];

    let tokenizer = init_tokenizer(None, Some(512), true).unwrap();

    c.bench_function("sent_tokenize_benchmark_paral_batch", |b| {
        b.iter(|| {
            let sentences = icu_sentence_tokenize(&data).unwrap();
            let sentences_text: Vec<_> = sentences.iter().map(|s| s.text.as_str()).collect();
            let encodings = tokenizer.encode_batch_fast(sentences_text, false).unwrap();
            black_box(encodings);
        })
    });
}

pub fn segment_sentences_benchmark(c: &mut Criterion) {
    let data_path = "tests/test_data/superlinear.txt";
    //let data_path = "tests/test_data/United_States.txt";
    //let data_path = "tests/test_data/bert_paper_complete.txt";

    let data = fs::read_to_string(data_path).unwrap();

    c.bench_function("segment_sentences_benchmark", |b| {
        b.iter(|| {
            let sentences = icu_sentence_tokenize(&data).unwrap();
            black_box(sentences);
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
        vec!["<ICU_SENT>".to_string()],
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
    //ws_tree_split_lite_benchmark(&mut criterion);
    //none_tree_split_lite_benchmark(&mut criterion);
    //hf_merged_tree_lite_sub_res_benchmark(&mut criterion);
    //ws_tokenize_words_benchmark(&mut criterion);
    segment_sentences_benchmark(&mut criterion);
    sent_tokenize_benchmark(&mut criterion);
    //sent_tokenize_benchmark_paral_batch(&mut criterion);
    sent_tik_tokenize_benchmark(&mut criterion);
    sent_kitoken_tokenize_benchmark(&mut criterion);
    sent_kitoken_simple_splits_benchmark(&mut criterion);
}

criterion_main!(benches);
