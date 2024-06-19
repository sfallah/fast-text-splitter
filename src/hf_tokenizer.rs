use tokenizers::normalizers::BertNormalizer;
#[cfg(feature = "tokenizers")]
use tokenizers::{Encoding, PaddingStrategy, Tokenizer, TruncationStrategy};
use tokenizers::{PaddingParams, TruncationParams};

use crate::common::{TokensResultLite, TokensResults};
use crate::encodings::EncodingType;
use crate::{Split, Tokenize};

#[cfg(feature = "tokenizers")]
pub fn init_tokenizer(
    model_path: Option<String>,
    max_len: Option<usize>,
    disable_normalizer: bool,
) -> tokenizers::Result<Tokenizer> {
    let default_model = "sentence-transformers/all-MiniLM-L6-v2".to_string();
    let model = match model_path {
        Some(model_path) => {
            if model_path.is_empty() {
                default_model
            } else {
                model_path
            }
        }
        None => default_model,
    };
    let mut tokenizer = Tokenizer::from_pretrained(model, None)?;
    tokenizer
        .get_padding_mut()
        .unwrap_or(&mut PaddingParams::default())
        .strategy = PaddingStrategy::BatchLongest;
    let tokenizer_truncation = tokenizer.get_truncation_mut();
    match tokenizer_truncation {
        None => {
            let mut truncation = TruncationParams::default();
            truncation.max_length = max_len.unwrap_or(40000);
            truncation.strategy = TruncationStrategy::LongestFirst;
            tokenizer
                .with_truncation(Option::from(truncation))
                .expect("TODO: panic message");
        }
        Some(truncation) => {
            truncation.max_length = max_len.unwrap_or(40000);
            truncation.strategy = TruncationStrategy::LongestFirst;
        }
    }
    if disable_normalizer {
        let normalizer = BertNormalizer::new(false, false, None, false);
        tokenizer.with_normalizer(normalizer);
    }
    Ok(tokenizer)
}

#[cfg(feature = "tokenizers")]
pub struct HFTokenizer {
    pub tokenizer: Tokenizer,
}

#[cfg(feature = "tokenizers")]
impl Tokenize for HFTokenizer {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType> {
        let encoded = self.tokenizer.encode(data, false).unwrap();
        Ok(EncodingType::HFEncoding(encoded))
    }
}

#[cfg(feature = "tokenizers")]
pub fn divide_encoding(encoded: &Encoding, splits: &[Split]) -> Vec<TokensResults> {
    let mut results = Vec::new();
    for split in splits {
        let result = {
            //if split.tokens_span.is_empty() {
            //TokensResults::default()
            //} else {
            let split_range = split.tokens_span.range();
            let ids = encoded.get_ids()[split_range.clone()].to_vec();
            let type_ids = encoded.get_type_ids()[split_range.clone()].to_vec();
            let attention_mask = encoded.get_attention_mask()[split_range.clone()].to_vec();
            let offsets = encoded.get_offsets()[split_range.clone()].to_vec();
            let tokens = encoded.get_tokens()[split_range.clone()].to_vec();

            TokensResults {
                ids,
                type_ids,
                attention_mask,
                offsets,
                tokens,
            }
        };
        results.push(result);
    }
    results
}

#[cfg(feature = "tokenizers")]
pub fn divide_encoding_lite<'a>(
    encoded: &'a Encoding,
    splits: &[Split],
) -> Vec<TokensResultLite<'a>> {
    let mut results = Vec::new();
    for split in splits {
        let result = {
            let ids = &encoded.get_ids()[split.tokens_span.range()];
            let offsets = &encoded.get_offsets()[split.tokens_span.range()];

            TokensResultLite {
                data_span: split.data_span.unwrap().clone(),
                ids,
                offsets,
            }
        };
        results.push(result);
    }
    results
}

#[cfg(feature = "tokenizers")]
#[cfg(test)]
mod tests {
    use super::*;
    use aho_corasick::Span;
    use tokenizers::pre_tokenizers::punctuation::Punctuation;
    use tokenizers::pre_tokenizers::sequence::Sequence;
    use tokenizers::pre_tokenizers::whitespace::WhitespaceSplit;
    use tokenizers::{
        OffsetReferential, OffsetType, PreTokenizedString, PreTokenizer, PreTokenizerWrapper,
    };

    #[test]
    fn tokens_len_test() -> tokenizers::Result<()> {
        let tokenizer = init_tokenizer(None, None, false)?;
        let data = "This is a test";
        let encoded = tokenizer.encode(data, false).unwrap();
        assert_eq!(encoded.len(), 4);
        Ok(())
    }

    #[test]
    fn tokens_len_test2() -> tokenizers::Result<()> {
        let tokenizer = init_tokenizer(None, None, false)?;
        let df_data = "\"You get out,\" I heard a thousand times, \"what you put in.\" I'm not sure, I don't think so.";
        let encoded = tokenizer.encode(df_data, false).unwrap();
        println!("tokens_no: {:?}", encoded.len());
        encoded
            .get_tokens()
            .iter()
            .for_each(|token| println!("{:?}", token));
        Ok(())
    }

    #[test]
    fn tokens_german_test() -> tokenizers::Result<()> {
        let model = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2".to_string();
        let tokenizer = init_tokenizer(Some(model), None, false)?;
        // german text with umlauts
        let data = "Lächeln ist die kürzeste Entfernung zwischen zwei Menschen. Über den Wolken muss die Freiheit wohl grenzenlos sein. Die Schüler lernen, wie man präzise Lösungen für schwierige Aufgaben findet. In München gibt es viele schöne Plätze zum Verweilen. Äpfel und Birnen wachsen im Garten.";
        let encoded = tokenizer.encode(data, false).unwrap();
        println!("tokens_no: {:?}", encoded.len());
        encoded
            .get_tokens()
            .iter()
            .zip(encoded.get_offsets().iter())
            .for_each(|(token, offset)| {
                println!("Offset: {:?}", offset);
                println!("Token: {:?}", token);
                println!("Data: {:?}", &data[offset.0..offset.1]);
            });
        Ok(())
    }

    #[test]
    fn tokens_arabic_test() -> tokenizers::Result<()> {
        let model = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2".to_string();
        let tokenizer = init_tokenizer(Some(model), None, false)?;
        let data = "مرحبا، كيف حالك؟";
        let encoded = tokenizer.encode(data, false).unwrap();
        assert_eq!(encoded.len(), 8);
        encoded
            .get_tokens()
            .iter()
            .for_each(|token| println!("{:?}", token));
        encoded
            .get_offsets()
            .iter()
            .for_each(|offset| println!("{:?}", offset));
        Ok(())
    }

    #[test]
    fn hf_pre_tokenizer_test() -> tokenizers::Result<()> {
        let pretokenizers = vec![
            PreTokenizerWrapper::WhitespaceSplit(WhitespaceSplit),
            PreTokenizerWrapper::Punctuation(Punctuation::default()),
        ];
        let pretok = Sequence::new(pretokenizers);
        let mut pretokenized: PreTokenizedString = "Hey friend!     How are you?!?".into();
        pretok.pre_tokenize(&mut pretokenized).unwrap();
        let offsets: Vec<_> = pretokenized
            .get_splits(OffsetReferential::Original, OffsetType::Byte)
            .into_iter()
            .map(|(s, o, _)| (s, o))
            .collect();
        for (s, o) in offsets {
            println!("{:?} {:?}", s, o);
        }
        Ok(())
    }

    #[test]
    fn vec_trivial_tests() {
        let vec = vec![1, 2, 3, 4, 5];
        let arr = vec.as_slice();
        let sub_arr = arr[0..0].to_vec();
        assert_eq!(sub_arr.len(), 0);

        let sub_arr = arr[0..1].to_vec();
        assert_eq!(sub_arr.len(), 1);
        assert_eq!(sub_arr[0], 1);

        let sub_arr = arr[4..5].to_vec();
        assert_eq!(sub_arr.len(), 1, "arr: {:?}", sub_arr);

        let sub_arr = arr[3..3].to_vec();
        assert_eq!(sub_arr.len(), 0, "arr: {:?}", sub_arr);

        let span = Span { start: 0, end: 0 };
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());

        let span = Span { start: 10, end: 10 };
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());

        let mut start = 0;

        loop {
            if start + 1 >= vec.len() {
                break;
            }
            start += 1;
        }
        assert_eq!(start, 4);
    }
}
