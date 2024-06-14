use aho_corasick::Span;
use fast_text_splitter::common::Split;
use fast_text_splitter::hf_tokenizer::init_tokenizer;
use fast_text_splitter::split_tokens_len;
use tokenizers::Encoding;

#[test]
fn test_tokens_len_slip() -> tokenizers::Result<()> {
    let data = "In fact, the correlation between superlinear returns and inequality is so strong that it yields another heuristic for finding work of this type: look for fields where a few big winners outperform everyone else. A kind of work where everyone does about the same is unlikely to be one with superlinear returns.\n\n";
    let tokenizer = init_tokenizer(None, None, false)?;
    let encoded = tokenizer.encode(data, false).unwrap();
    assert_eq!(encoded.len(), 64);
    encoded.get_tokens().iter().for_each(|t| println!("{}", t));
    let multi_tokens_word_split = split_tokens_len(
        &encoded.get_word_ids(),
        Span {
            start: 0,
            end: encoded.len(),
        },
        60,
    );
    assert_eq!(multi_tokens_word_split.len(), 2);
    assert_eq!(multi_tokens_word_split[0].tokens_span.len(), 59);
    assert_eq!(multi_tokens_word_split[1].tokens_span.len(), 5);
    print_len_splits(&encoded, &multi_tokens_word_split);

    let clean_cut_split = split_tokens_len(
        &encoded.get_word_ids(),
        Span {
            start: 0,
            end: encoded.len(),
        },
        40,
    );
    print_len_splits(&encoded, &clean_cut_split);

    Ok(())
}

#[test]
fn test_tokens_len_slip_2() -> tokenizers::Result<()> {
    let data = "العربيةAragonésБеларускаяCatalàDanskDeutschEspañolEsperantoفارسیFrançaisՀայերենहिन्दीHrvatskiBahasa IndonesiaItalianoҚазақшаMagyarNederlands日本語NorskNorsk nynorskPolskiPortuguêsРусскийTürkçeУкраїнськаTiếng Việt中文 ";

    let model = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2".to_string();
    let tokenizer = init_tokenizer(Some(model), None, false)?;
    let encoded = tokenizer.encode(data, false).unwrap();
    println!("no-tokens: {:?}", encoded.len());

    let max_tokens: usize = 55;

    let multi_tokens_word_split = split_tokens_len(
        &encoded.get_word_ids(),
        Span {
            start: 0,
            end: encoded.len(),
        },
        max_tokens,
    );
    print_len_splits(&encoded, &multi_tokens_word_split);
    assert_eq!(multi_tokens_word_split.len(), 2);
    assert_eq!(multi_tokens_word_split[0].tokens_span.len(), 46);
    assert_eq!(multi_tokens_word_split[1].tokens_span.len(), 18);

    Ok(())
}

fn print_len_splits(encoded: &Encoding, multi_tokens_word_split: &Vec<Split>) {
    println!("{:?}", encoded.get_tokens().to_vec());
    for split in multi_tokens_word_split.iter() {
        println!("{:?}", split.tokens_span);
        let split_tokens =
            &encoded.get_tokens()[split.tokens_span.start..split.tokens_span.end].to_vec();
        println!("{:?}", split_tokens);
    }
}
