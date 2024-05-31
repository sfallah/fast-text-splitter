use std::fs;

use itertools::Itertools;

use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::hf_tokenizer::init_tokenizer;
#[cfg(feature = "tokenizers")]
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::text_split_parallel;
use fast_text_splitter::ws_tokenizer::{
    whitespace_punctuation_tokenize, ws_punc_tokens, WSTokenizer,
};

#[test]
#[cfg(feature = "tokenizers")]
fn hf_parallel_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you foo and poo? \n\n I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::hf_default();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn ws_parallel_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you ? \n\n I am fine. \n Nice to meet you all insecure!";

    let conf_params = ConfigParams::builder()
        .max_tokens(6)
        .max_depth(2)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split.tokens_span.len());
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn no_matches_ws_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::ws_default();
    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);
    let splits = text_split_parallel(&conf, data);
    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn no_matches_hf_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::hf_default();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 1);
    assert_eq!(splits[0].split_strings, data);

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn dot_pattern_hf_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::builder()
        .pattern(vec![vec![".".to_string()]])
        .max_depth(1)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 2);

    splits
        .iter()
        .for_each(|split| println!("{:?}", split.split_strings));

    let expected_splits = vec![
        "Hello, you all! How are you ? I am fine.",
        " Nice to meet you all insecure!",
    ];
    for (split, expected) in splits.iter().zip(expected_splits.iter()) {
        assert_eq!(split.split_strings, *expected);
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn no_match_max_hf_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 3);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let expected_splits = vec![
        "Hello, you all! How are you ",
        "? I am fine. Nice to meet ",
        "you all insecure!",
    ];
    for (split, expected) in splits.iter().zip(expected_splits.iter()) {
        assert_eq!(split.split_strings, *expected);
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn end_match_hf_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure! \n\n";

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 3);

    let _total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    //assert_eq!(total_len, data.len());

    let expected_splits = vec![
        "Hello, you all! How are you ",
        "? I am fine. Nice to meet ",
        "you all insecure! \n\n",
    ];
    for (split, expected) in splits.iter().zip(expected_splits.iter()) {
        assert_eq!(split.split_strings, *expected);
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn first_match_hf_test() -> tokenizers::Result<()> {
    let data = "\n\n Hello, you all! How are you ? I am fine. Nice to meet you all insecure.";

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .merge_level(0)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 3);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let expected_splits = vec![
        "\n\n Hello, you all! How are you ",
        "? I am fine. Nice to meet ",
        "you all insecure.",
    ];
    for (split, expected) in splits.iter().zip(expected_splits.iter()) {
        assert_eq!(split.split_strings, *expected);
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn nl_match_hf_test() -> tokenizers::Result<()> {
    let data = "\n\n Hello, you all! How are you ? \n I am fine. Nice to meet you all insecure! \n \n \n\n";

    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec!["!".to_string(), "?".to_string(), ".".to_string()],
        ])
        .max_tokens(12)
        .max_depth(3)
        .merge_level(0)
        .parallel(false)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 2);

    splits
        .iter()
        .for_each(|split| println!("{:?}", split.split_strings));

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let expected_splits = vec![
        "\n\n Hello, you all! How are you ? \n ",
        "I am fine. Nice to meet you all insecure! \n \n \n\n",
    ];
    for (split, expected) in splits.iter().zip(expected_splits.iter()) {
        assert_eq!(split.split_strings, *expected);
    }

    Ok(())
}

#[test]
fn no_match_max_ws_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .pattern(vec![vec!["\n\n".to_string()], vec!["\n".to_string()]])
        .parallel(true)
        .build();
    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    let splits = text_split_parallel(&conf, data);
    assert_eq!(splits.len(), 3);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let expected_splits = vec![
        "Hello, you all! How are you ",
        "? I am fine. Nice to meet ",
        "you all insecure!",
    ];

    for (split, expected) in splits.iter().zip(expected_splits.iter()) {
        assert_eq!(split.split_strings, *expected);
    }

    Ok(())
}

#[test]
fn split_words_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";

    let bytes = fs::read(data_path)?;
    let data = std::str::from_utf8(&bytes).unwrap();

    //let conf_params = ConfigParams::ws_default();
    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(40000)
        //.max_tokens(60)
        .merge_level(0)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    let splits = text_split_parallel(&conf, data);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
        let tokens = ws_punc_tokens(&split.split_strings);
        assert_eq!(
            tokens.len(),
            split.split.tokens_span.len(),
            "\n Split String: {:?} \n Tokens: {:?}",
            split.split_strings,
            tokens
        );
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn split_tokenizer_encoding_error_test() -> tokenizers::Result<()> {
    let data_path = "tests/error_data/Selena Gomez - Wikipedia.txt";

    let bytes = fs::read(data_path)?;
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf_params = ConfigParams::builder()
        .merge_level(1)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
    }
    Ok(())
}

#[test]
fn sentence_pattern_ws_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you? \n\n I am fine. \n Nice to meet you all!";

    // Test custom configuration with different patterns
    let conf_params_patterns = ConfigParams::builder()
        .max_tokens(5)
        .max_depth(2)
        .parallel(false)
        .build();

    let conf_patterns = SplitterConfig::<WSTokenizer>::from_params(&conf_params_patterns);
    let splits_results = text_split_parallel(&conf_patterns, data);

    println!("Custom Configuration Splits:");
    for split in splits_results.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

fn vec_dedup<T: Eq + std::hash::Hash + Clone>(vec: Vec<T>) -> Vec<T> {
    vec.into_iter().unique().collect()
}

#[test]
fn vec_dedup_test() {
    let vec = vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5];
    let dedup_vec = vec_dedup(vec);
    assert_eq!(dedup_vec, vec![1, 2, 3, 4, 5, 6]);

    let opt_vec = vec![
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
    ];
    let dedup_opt_vec = vec_dedup(opt_vec);
    assert_eq!(
        dedup_opt_vec,
        vec![Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)]
    );
}

#[test]
fn word_tokens_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you? \n\n I am fine. \n Nice to meet you all!";

    let hf_tokenizer = init_tokenizer(None, Some(40000), true)?;
    let hf_encoding = hf_tokenizer.encode(data, false)?;
    let hf_word_ids: Vec<_> = vec_dedup(hf_encoding.get_word_ids().to_vec());

    let ws_token_offsets = whitespace_punctuation_tokenize(data);
    assert_eq!(hf_word_ids.len(), ws_token_offsets.len());

    Ok(())
}

#[test]
fn word_tokens_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";
    let binding = fs::read_to_string(data_path).unwrap();
    let data = binding.as_str();

    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string()],
        ])
        .tokenizer_max_len(40000)
        .max_tokens(60)
        .max_depth(0)
        .merge_level(1)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    let splits = text_split_parallel(&conf, data);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let hf_tokenizer = init_tokenizer(None, Some(40000), true)?;

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
        let tokens = ws_punc_tokens(&split.split_strings);
        assert_eq!(
            tokens.len(),
            split.split.tokens_span.len(),
            "\n Split String: {:?} \n Tokens: {:?}",
            split.split_strings,
            tokens
        );
        let hf_encoding = hf_tokenizer.encode(split.split_strings.as_str(), false)?;
        let hf_word_ids: Vec<_> = vec_dedup(hf_encoding.get_word_ids().to_vec());
        assert_eq!(hf_word_ids.len(), tokens.len());
    }

    Ok(())
}

#[test]
fn hf_tokenize_batch_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";
    let binding = fs::read_to_string(data_path).unwrap();
    let data = binding.as_str();

    let conf = SplitterConfig::<WSTokenizer>::from_params(&ConfigParams::ws_default());

    let splits = text_split_parallel(&conf, data);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    tokenizers::utils::parallelism::set_parallelism(true);

    let hf_tokenizer = init_tokenizer(None, Some(40000), true)?;

    let split_str: Vec<_> = splits
        .iter()
        .map(|split| split.split_strings.clone())
        .collect();

    let encoded = hf_tokenizer.encode_batch(split_str, false)?;

    splits
        .iter()
        .zip(encoded.iter())
        .for_each(|(split, encode)| {
            let ids_len = encode.get_ids().len();
            let token_ids: Vec<u32> =
                if let Some(pos) = encode.get_ids().iter().rev().position(|&id| id != 0) {
                    encode.get_ids()[..ids_len - pos].to_vec()
                } else {
                    encode.get_ids().to_vec()
                };

            let act_enc_len = token_ids.len();
            let hf_word_ids: Vec<_> = vec_dedup(
                encode
                    .get_word_ids()
                    .iter()
                    .filter(|opt| opt.is_some())
                    .collect(),
            );

            println!(
                "Split Len: {}, No-tokens: {}, HF Words: {:?}",
                split.split.tokens_span.len(),
                act_enc_len,
                hf_word_ids.len()
            );
            println!("{:?}", token_ids);
            println!("{:?}", encode.get_tokens()[..act_enc_len].to_vec());

            let ws_words = ws_punc_tokens(&split.split_strings);

            assert_eq!(
                hf_word_ids.len(),
                split.split.tokens_span.len(),
                "\n Split String: {:?} \n HF-Words: {:?} \n WS-Words: {:?}",
                split.split_strings,
                encode.get_word_ids(),
                ws_words
            );
        });

    Ok(())
}
