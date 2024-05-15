use crate::common::TokensResults;
use crate::{Split, Tokenize};

#[cfg(feature = "tokenizers")]
use tokenizers::{Encoding, PaddingStrategy, Tokenizer, TruncationStrategy};
use crate::encodings::EncodingType;

#[cfg(feature = "tokenizers")]
pub fn init_tokenizer(model: Option<String>) -> tokenizers::Result<Tokenizer> {
    let model = model.unwrap_or("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let mut tokenizer = Tokenizer::from_pretrained(model, None)?;
    tokenizer.get_padding_mut().unwrap().strategy = PaddingStrategy::BatchLongest;
    let tokenizer_truncation = tokenizer.get_truncation_mut().unwrap();
    tokenizer_truncation.strategy = TruncationStrategy::LongestFirst;
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
