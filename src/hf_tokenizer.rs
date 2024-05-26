#[cfg(feature = "tokenizers")]
use tokenizers::{Encoding, PaddingStrategy, Tokenizer, TruncationStrategy};
use tokenizers::{PaddingParams, TruncationParams};

use crate::common::TokensResults;
use crate::encodings::EncodingType;
use crate::{Split, Tokenize};

#[cfg(feature = "tokenizers")]
pub fn init_tokenizer(
    model: Option<String>,
    max_len: Option<usize>,
) -> tokenizers::Result<Tokenizer> {
    let default_model = "sentence-transformers/all-MiniLM-L6-v2".to_string();
    let model = match model {
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
            truncation.max_length = max_len.unwrap_or(5120);
            truncation.strategy = TruncationStrategy::LongestFirst;
            tokenizer
                .with_truncation(Option::from(truncation))
                .expect("TODO: panic message");
        }
        Some(truncation) => {
            truncation.max_length = max_len.unwrap_or(5120);
            truncation.strategy = TruncationStrategy::LongestFirst;
        }
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
    let mut start = 0;
    for split in splits {
        let ids = encoded.get_ids()[start..split.tokens_span.end].to_vec();
        let type_ids = encoded.get_type_ids()[start..split.tokens_span.end].to_vec();
        let attention_mask = encoded.get_attention_mask()[start..split.tokens_span.end].to_vec();
        let offsets = encoded.get_offsets()[start..split.tokens_span.end].to_vec();
        results.push(TokensResults {
            ids,
            type_ids,
            attention_mask,
            offsets,
        });
        start = split.tokens_span.end;
    }
    results
}

#[cfg(feature = "tokenizers")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_len_test() -> tokenizers::Result<()> {
        let tokenizer = init_tokenizer(None, None)?;
        let data = "This is a test";
        let encoded = tokenizer.encode(data, false).unwrap();
        assert_eq!(encoded.len(), 4);
        Ok(())
    }

    #[test]
    fn tokens_len_test2() -> tokenizers::Result<()> {
        let tokenizer = init_tokenizer(None, None)?;
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
        let tokenizer = init_tokenizer(Some(model), None)?;
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
        let tokenizer = init_tokenizer(Some(model), None)?;
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
}
