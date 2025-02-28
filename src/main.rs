use std::fs;
use std::str::from_utf8;
use fast_text_splitter::config::SplitterLiteConfig;
use fast_text_splitter::hf_tokenizer::init_tokenizer;

fn main() {
    let data_path = "tests/test_data/superlinear.txt";
    let binding = fs::read_to_string(data_path).unwrap();
    let data = binding.as_bytes();
    let patterns = vec![
        vec!["\n\n".to_string()],
        vec!["\n".to_string()],
        vec![".".to_string(), "!".to_string(), "?".to_string()],
    ];

    let splitter_config = SplitterLiteConfig::new_hf(patterns, Some(512), None, true, None);
    let splits = splitter_config.hf_splits(data);


    println!("number of splits: {:?}", splits.len());
    for split in splits.iter() {
        println!("{:?}", split.split_string);
        println!("{:?}", split.split_string.len());
        println!("{:?}", split.tokens.len());

    }
}