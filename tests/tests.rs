use std::fs;

use fast_text_splitter::config::{ConfigParams, SplitterConfig};
#[cfg(feature = "tokenizers")]
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::text_split_parallel;
use fast_text_splitter::ws_tokenizer::WSTokenizer;

#[test]
#[cfg(feature = "tokenizers")]
fn hf_parallel_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you foo and poo? \n\n I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::hf_default();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data, None);

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

    let splits = text_split_parallel(&conf, data, None);

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

    let splits = text_split_parallel(&conf, data, None);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);
    let splits = text_split_parallel(&conf, data, None);
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

    let splits = text_split_parallel(&conf, data, None);
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

    let splits = text_split_parallel(&conf, data, None);
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

    let splits = text_split_parallel(&conf, data, None);
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

    let splits = text_split_parallel(&conf, data, None);
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
    let data = "\n\n Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data, None);
    assert_eq!(splits.len(), 3);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let expected_splits = vec![
        "\n\n Hello, you all! How are you ",
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
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data, None);
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

    let splits = text_split_parallel(&conf, data, None);
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
            vec![".".to_string()],
        ])
        .tokenizer_max_len(40000)
        .max_tokens(256)
        .max_depth(2)
        .merge_level(1)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<WSTokenizer>::from_params(&conf_params);

    let splits = text_split_parallel(&conf, data, None);

    //let total_len = splits.iter().map(|split| split.split_strings.len()).sum::<usize>();
    //assert_eq!(total_len, data.len());

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn split_tokenizer_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";

    let bytes = fs::read(data_path)?;
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf_params = ConfigParams::builder()
        .merge_level(1)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data, None);

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
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

    let splits = text_split_parallel(&conf, data, None);

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
    let splits_results = text_split_parallel(&conf_patterns, data, None);

    println!("Custom Configuration Splits:");
    for split in splits_results.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}
