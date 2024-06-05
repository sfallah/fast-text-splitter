use fast_text_splitter::common::Split;
use fast_text_splitter::config::{ConfigParams, SplitterConfig};
use fast_text_splitter::encodings::Tokenize;
use fast_text_splitter::hf_tokenizer::HFTokenizer;
use fast_text_splitter::span;

#[test]
fn test_encodings_tests() {
    let data = "In fact, the correlation. Between superlinear.\n";

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
    let encoded = conf.tokenizer.encode(data).unwrap();
    let tokens_spans = vec![span(0, 6), span(6, 11)];

    let data_spans = encoded.to_data_offsets(tokens_spans.clone(), span(0, data.len()));

    let splits: Vec<_> = data_spans
        .iter()
        .zip(tokens_spans.iter())
        .map(|(data_span, tokens_span)| Split {
            tokens_span: tokens_span.clone(),
            data_span: Some(data_span.clone()),
        })
        .collect();
    let split_results = encoded.to_split_results(&splits, data);

    for split_result in split_results.iter() {
        println!("{:?}", split_result.split);
        println!("{:?}", split_result.split_strings);
        println!("{:?}", split_result.results.clone().unwrap().tokens);
    }
}
