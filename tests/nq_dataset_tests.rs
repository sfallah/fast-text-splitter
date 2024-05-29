extern crate unicode_normalization;

use std::{fs, io};

use tokenizers::Tokenizer;

use fast_text_splitter::common::SplitResults;
use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::hf_tokenizer::{init_tokenizer, HFTokenizer};
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

fn tokenize_data(
    conf: &SplitterConfig<HFTokenizer>,
    data: &str,
    normalize: bool,
) -> io::Result<Vec<SplitResults>> {
    let splits = text_split_parallel(&conf, data, Some(normalize));
    Ok(splits)
}

fn tokenize_file(
    conf: &SplitterConfig<HFTokenizer>,
    tokenizer: &Tokenizer,
    file: &str,
    data_len_check: bool,
    normalize: bool,
    check_splits: bool,
) -> io::Result<()> {
    println!("### Tokenize file: {}", file);
    let data = fs::read_to_string(file)?;
    tokenize_file_data(
        conf,
        tokenizer,
        data_len_check,
        &data,
        normalize,
        check_splits,
        file,
    )?;
    Ok(())
}

fn tokenize_file_data(
    conf: &SplitterConfig<HFTokenizer>,
    tokenizer: &Tokenizer,
    data_len_check: bool,
    in_data: &String,
    normalize: bool,
    check_splits: bool,
    file: &str,
) -> io::Result<()> {
    println!("    Content-length: {}", in_data.len());

    let hf_encoding = tokenizer.encode(in_data.clone(), false).unwrap();
    println!("    Number of tokens: {}", hf_encoding.len());

    let splits = tokenize_data(conf, &in_data, normalize)?;
    println!("    Number of Splits: {}", splits.len());

    let mut reconstructed_data = String::new();

    for split in splits.iter() {
        reconstructed_data.push_str(split.split_strings.as_str());
    }

    println!("    Total data length: {}", reconstructed_data.len());

    if data_len_check {
        assert_eq!(reconstructed_data.len(), in_data.len(), "File: {}", file);
        //assert_eq!(reconstructed_data, in_data.clone());
    }

    if check_splits {
        for split in splits.iter() {
            let split_str_encoded = tokenizer
                .encode(split.split_strings.clone(), false)
                .unwrap();
            let split_str_encoded_len = split_str_encoded.len();
            assert_eq!(
                split_str_encoded_len,
                split.results.clone().unwrap().ids.len(),
                "File: {} \n Split-text: {:?} ",
                file,
                split.split_strings
            );
        }
    }

    let total_tk_results = splits
        .iter()
        .map(|split| split.results.clone().unwrap().ids.len())
        .sum::<usize>();

    println!("    Total tokens results: {}", total_tk_results);
    assert_eq!(hf_encoding.len(), total_tk_results);

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
    //let conf_params = ConfigParams::builder().tokenizer_max_len(40000).build();
    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string()],
        ])
        .tokenizer_max_len(40000)
        .max_tokens(30)
        .max_depth(3)
        .merge_level(1)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);
    let tokenizer = init_tokenizer(None, Some(40000))?;
    for file in files.iter() {
        tokenize_file(&conf, &tokenizer, file, true, false, true)?;
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn text_normalize_test() -> tokenizers::Result<()> {
    //let file = "data/dev/Super Bowl 50 halftime show - Wikipedia.txt";
    //let file = "data/dev/List_of_Orange_Is_the_New_Black_characters.txt";
    let file = "tests/error_data/Geothermal_gradient.txt";
    let data = fs::read_to_string(file)?;

    let tokenizer = init_tokenizer(None, Some(data.len()))?;

    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(data.len())
        .max_tokens(50)
        .merge_level(0)
        .max_depth(2)
        //.parallel(true)
        .build();

    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);
    tokenize_file_data(&conf, &tokenizer, false, &data, true, true, file)?;

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_test() -> tokenizers::Result<()> {
    let files = list_text_files("data/dev/")?;
    let tokenizer = init_tokenizer(None, Some(100_000))?;

    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(100_000)
        .merge_level(1)
        .max_depth(3)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);


    files.par_iter().for_each(|file| {
        tokenize_file(&conf, &tokenizer, file, true, false, false).unwrap();
    });

    Ok(())
}
