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

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.splits);
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn ws_parallel_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! \n How are you ? \n\n I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::ws_default();
    let conf = SplitterConfig::<WSTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}

#[test]
fn no_matches_ws_splits_test() -> tokenizers::Result<()> {
    let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

    let conf_params = ConfigParams::ws_default();
    let conf = SplitterConfig::<WSTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<WSTokenizer>::from_params(conf_params);
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

    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    let conf_params = ConfigParams::builder()
        .max_tokens(8)
        .max_depth(2)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);
    for split in splits.iter() {
        println!("{:?}", split.split_strings);
    }

    Ok(())
}
#[test]
fn split_words_superlinear_test() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";

    let bytes = fs::read(data_path)?;
    let data = std::str::from_utf8(&bytes).unwrap();

    let conf_params = ConfigParams::ws_default();
    let conf = SplitterConfig::<WSTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.splits);
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
        .max_tokens(68)
        .max_depth(2)
        .parallel(true)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    for split in splits.iter() {
        println!("{:?}", split.splits);
        println!("{:?}", split.split_strings);
    }
    Ok(())
}
