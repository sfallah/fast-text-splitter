use std::{fs, io};
use fast_text_splitter::common::SplitResults;
use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::text_split_parallel;
use rayon::prelude::*;

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

fn tokenize_data(data: &str) -> io::Result<Vec<SplitResults>> {
    let conf_params = ConfigParams::hf_default();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);
    let splits = text_split_parallel(&conf, data);
    Ok(splits)
}

fn tokenize_file(file: &str) -> io::Result<()> {
    println!("### Tokenize file: {}", file);
    let data = fs::read_to_string(file)?;
    println!("    Content-length: {}", data.len());
    let splits = tokenize_data(&data)?;
    println!("    Number of Splits: {}", splits.len());
    let total_data_len = splits.iter().map(|split| split.split_strings.len()).sum::<usize>();
    println!("    Total data length: {}", total_data_len);
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
    for file in files.iter() {
        tokenize_file(file)?;
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_test() -> tokenizers::Result<()> {
    let files = list_text_files("data/")?;
    for file in files.iter() {
        tokenize_file(file)?;
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_par_test() -> tokenizers::Result<()> {
    let files = list_text_files("data/")?;

    files.par_iter().for_each(|file| {
        tokenize_file(file).unwrap();
    });

    Ok(())
}