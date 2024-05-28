use fast_text_splitter::common::SplitResults;
use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::hf_tokenizer::{init_tokenizer, HFTokenizer};
use fast_text_splitter::text_split_parallel;
use std::{fs, io};
use tokenizers::Tokenizer;

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

fn tokenize_file(
    conf: &SplitterConfig<HFTokenizer>,
    tokenizer: &Tokenizer,
    file: &str,
    data_len_check: bool,
) -> io::Result<()> {
    println!("### Tokenize file: {}", file);
    let data = fs::read_to_string(file)?;
    let hf_encoding = tokenizer.encode(data.to_string(), false).unwrap();
    println!("    Content-length: {}", data.len());
    println!("    Number of tokens: {}", hf_encoding.len());
    let splits = tokenize_data(conf, &data)?;
    println!("    Number of Splits: {}", splits.len());
    let total_data_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    println!("    Total data length: {}", total_data_len);
    if data_len_check {
        assert_eq!(total_data_len, data.len());
    }
    let total_tokens_len = splits
        .iter()
        .map(|split| split.split.no_tokens())
        .sum::<usize>();
    println!("    Total tokens length: {}", total_tokens_len);
    assert_eq!(hf_encoding.len(), total_tokens_len);

    let total_tk_results = splits
        .iter()
        .map(|split| split.results.as_ref().unwrap().ids.len())
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
    let conf_params = ConfigParams::builder().tokenizer_max_len(40000).build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);
    let tokenizer = init_tokenizer(None, Some(40000))?;
    for file in files.iter() {
        tokenize_file(&conf, &tokenizer, file, false)?;
    }
    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn hf_nq_dataset_test() -> tokenizers::Result<()> {
    let files = list_text_files("data/dev/")?;
    let tokenizer = init_tokenizer(None, Some(400000))?;
    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(400000)
        .merge_level(1)
        .max_depth(3)
        .build();
    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    for file in files.iter() {
        tokenize_file(&conf, &tokenizer, file, false)?;
    }
    Ok(())
}
