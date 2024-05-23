use std::{fs, io};
use fast_text_splitter::common::SplitResults;
use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::hf_tokenizer::{HFTokenizer, init_tokenizer};
use fast_text_splitter::text_split_parallel;
use rayon::prelude::*;
use tokenizers::{Tokenizer};

fn list_text_files(dir: &str) -> io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            files.push(String::from(path.to_str().unwrap()));
        }
    }

    Ok(files)
}

fn tokenize_data(conf: &SplitterConfig<HFTokenizer>, data: &str) -> io::Result<Vec<SplitResults>> {
    let splits = text_split_parallel(&conf, data);
    Ok(splits)
}

fn tokenize_file(conf: &SplitterConfig<HFTokenizer>, tokenizer: &Tokenizer, file: &str) -> io::Result<()> {
    println!("### Tokenize file: {}", file);
    let data = fs::read_to_string(file)?;
    let hf_encoding = tokenizer.encode(data.to_string(), false).unwrap();
    println!("    Number of tokens: {}", hf_encoding.len());
    println!("    Content-length: {}", data.len());
    let splits = tokenize_data(conf, &data)?;
    println!("    Number of Splits: {}", splits.len());
    let total_data_len = splits.iter().map(|split| split.split_strings.len()).sum::<usize>();
    println!("    Total data length: {}", total_data_len);
    let total_tokens_len = splits.iter().map(|split| split.splits.no_tokens()).sum::<usize>();
    println!("    Total tokens length: {}", total_tokens_len);
    Ok(())
}

#[test]
fn list_files_test() -> io::Result<()> {
    let files = list_text_files("tests/test_data/")?;

    let expected_files = vec![
        "tests/test_data/mit.txt".to_string(),
        "tests/test_data/superlinear.txt".to_string(),
    ];
    for file in files.iter() {
        println!("{}", file);
    }
    assert_eq!(files, expected_files);

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_local_data_test() -> tokenizers::Result<()> {
    let files = list_text_files("tests/test_data/")?;
    let conf_params = ConfigParams::builder().tokenizer_max_len(40000).build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);
    let tokenizer = init_tokenizer(None, Some(40000))?;
    for file in files.iter() {
        tokenize_file(&conf, &tokenizer, file)?;
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_test() -> tokenizers::Result<()> {
    let files = list_text_files("data/train/")?;
    let tokenizer = init_tokenizer(None, Some(40000))?;
    let conf_params = ConfigParams::builder().tokenizer_max_len(40000).build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    for file in files.iter() {
        tokenize_file(&conf, &tokenizer, file)?;
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_par_test() -> tokenizers::Result<()> {
    let files = list_text_files("data/")?;
    let tokenizer = init_tokenizer(None, Some(40000))?;
    let conf_params = ConfigParams::builder().tokenizer_max_len(40000).build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    files.par_iter().for_each(|file| {
        tokenize_file(&conf, &tokenizer, file).unwrap();
    });

    Ok(())
}