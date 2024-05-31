use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::hf_tokenizer::{init_tokenizer, HFTokenizer};
use fast_text_splitter::text_split_parallel;
use std::fs;

#[test]
#[cfg(feature = "tokenizers")]
fn split_tokenizer_simple_test() -> tokenizers::Result<()> {
    let data = "In fact, the correlation. Between superlinear.\n\
    Returns and inequality is so strong that it yields.\n\n\
    Another heuristic for.\n\
    Finding work of this type.\n\
    Look for fields where.\n\
    A few big winners. \n\
    Outperform everyone else.\n\n";

    //let conf_params = ConfigParams::ws_default();
    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(512)
        .max_tokens(20)
        .merge_level(0)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data);

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let hf_tokenizer = init_tokenizer(None, Some(40000), false)?;

    let data_encoded = hf_tokenizer.encode(data, false)?;

    let total_no_tokens = splits
        .iter()
        .map(|split| split.results.clone().unwrap().ids.len())
        .sum::<usize>();

    assert_eq!(total_no_tokens, data_encoded.len());

    let splits_check = true;

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
        println!("{:?}", split.results.clone().unwrap().tokens);

        if splits_check {
            let split_encoded = hf_tokenizer.encode(split.split_strings.as_str(), false)?;

            assert_eq!(
                split_encoded.len(),
                split.results.clone().unwrap().ids.len(),
                "\n Split String: {:?} \n Split Tokens: {:?} \n Result Tokens: {:?}",
                split.split_strings,
                split_encoded.get_tokens(),
                split.results.clone().unwrap().tokens
            );

            assert_eq!(
                split_encoded.get_tokens(),
                split.results.clone().unwrap().tokens
            );
        }
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn split_tokenizer_superlinear_test() -> tokenizers::Result<()> {
    //let data_path = "tests/error_data/superlinear_error.txt";
    let data_path = "tests/test_data/superlinear.txt";

    let data = fs::read_to_string(data_path).unwrap();

    //let conf_params = ConfigParams::ws_default();
    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(data.len())
        .max_tokens(60)
        .merge_level(0)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data.as_str());

    let total_len = splits
        .iter()
        .map(|split| split.split_strings.len())
        .sum::<usize>();
    assert_eq!(total_len, data.len());

    let hf_tokenizer = init_tokenizer(None, Some(40000), false)?;

    let data_encoded = hf_tokenizer.encode(data, false)?;

    let total_no_tokens = splits
        .iter()
        .map(|split| split.results.clone().unwrap().ids.len())
        .sum::<usize>();

    assert_eq!(total_no_tokens, data_encoded.len());

    let splits_check = true;

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
        println!("{:?}", split.results.clone().unwrap().tokens);

        if splits_check {
            let split_encoded = hf_tokenizer.encode(split.split_strings.as_str(), false)?;

            assert_eq!(
                split_encoded.len(),
                split.results.clone().unwrap().ids.len(),
                "\n Split String: {:?} \n Split Tokens: {:?} \n Result Tokens: {:?}",
                split.split_strings,
                split_encoded.get_tokens(),
                split.results.clone().unwrap().tokens
            );

            assert_eq!(
                split_encoded.get_tokens(),
                split.results.clone().unwrap().tokens
            );
        }
    }

    Ok(())
}

#[test]
#[cfg(feature = "tokenizers")]
fn split_tokenizer_superlinear_output() -> tokenizers::Result<()> {
    let data_path = "tests/test_data/superlinear.txt";

    let data = fs::read_to_string(data_path).unwrap();

    //let conf_params = ConfigParams::ws_default();
    let conf_params = ConfigParams::builder()
        .pattern(vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ])
        .tokenizer_max_len(data.len())
        .max_tokens(60)
        .merge_level(1)
        .parallel(true)
        .build();

    let conf = SplitterConfig::<HFTokenizer>::from_params(conf_params);

    let splits = text_split_parallel(&conf, data.as_str());

    for split in splits.iter() {
        println!("{:?}", split.split);
        println!("{:?}", split.split_strings);
    }
    Ok(())
}
