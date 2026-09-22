use crate::common::tokens_result_lite::TokensResultLite;
use aho_corasick::Span;
use tokenizers::normalizers::BertNormalizer;
use tokenizers::{Encoding, PaddingStrategy, Tokenizer, TruncationStrategy};
use tokenizers::{PaddingParams, TruncationParams};

use crate::encodings::{EncodingType, Tokenize};

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
            truncation.max_length = max_len.unwrap_or(usize::MAX);
            truncation.strategy = TruncationStrategy::LongestFirst;
            tokenizer
                .with_truncation(Option::from(truncation))
                .expect("TODO: panic message");
        }
        Some(truncation) => {
            truncation.max_length = max_len.unwrap_or(usize::MAX);
            truncation.strategy = TruncationStrategy::LongestFirst;
        }
    }
    if disable_normalizer {
        let normalizer = BertNormalizer::new(false, false, None, false);
        tokenizer.with_normalizer(Some(normalizer));
    }
    Ok(tokenizer)
}

pub struct HFTokenizer {
    pub tokenizer: Tokenizer,
}

impl Tokenize for HFTokenizer {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType> {
        let encoded = self.tokenizer.encode(data, false).unwrap();
        Ok(EncodingType::HFEncoding(HFEncoding {
            hf_encoding: encoded,
        }))
    }
}

pub struct HFEncoding {
    pub hf_encoding: Encoding,
}

impl HFEncoding {
    pub fn get_offsets(&self) -> &[(usize, usize)] {
        self.hf_encoding.get_offsets()
    }
    pub fn get_word_ids(&self) -> &[Option<u32>] {
        self.hf_encoding.get_word_ids()
    }
    pub fn len(&self) -> usize {
        self.hf_encoding.len()
    }
    pub fn is_empty(&self) -> bool {
        self.hf_encoding.is_empty()
    }

    pub fn divide_encoding_lite(&self, tokens_span: Option<Span>) -> TokensResultLite {
        if let Some(tokens_span) = tokens_span {
            let ids = &self.hf_encoding.get_ids()[tokens_span.range()];
            let offsets = &self.hf_encoding.get_offsets()[tokens_span.range()];

            TokensResultLite {
                ids: Some(ids.to_vec()),
                offsets: Some(offsets.to_vec()),
            }
        } else {
            TokensResultLite {
                ids: None,
                offsets: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{tokenizer, DEFAULT_MODEL, MULTILINGUAL_MODEL};

    #[test]
    fn default_model_token_count() {
        let encoded = tokenizer(DEFAULT_MODEL)
            .encode("This is a test", false)
            .unwrap();
        assert_eq!(encoded.len(), 4);
    }

    #[test]
    fn init_tokenizer_truncates_to_max_len() {
        let tokenizer = init_tokenizer(None, Some(3), false).unwrap();
        assert_eq!(tokenizer.encode("This is a test", false).unwrap().len(), 3);
    }

    #[test]
    fn multilingual_model_token_count() {
        let encoded = tokenizer(MULTILINGUAL_MODEL)
            .encode("مرحبا، كيف حالك؟", false)
            .unwrap();
        assert_eq!(encoded.len(), 8);
    }

    /// The splitter maps byte spans to token spans through the offsets, so they must be
    /// in-bounds char boundaries that never go backwards. They may overlap by one byte:
    /// a standalone `▁` token is given the first byte of the word that follows it.
    #[test]
    fn multilingual_offsets_are_ordered_char_boundaries() {
        let text = "Lächeln ist die kürzeste Entfernung zwischen zwei Menschen. \
            Über den Wolken muss die Freiheit wohl grenzenlos sein. \
            Die Schüler lernen, wie man präzise Lösungen für schwierige Aufgaben findet. \
            In München gibt es viele schöne Plätze zum Verweilen. \
            Äpfel und Birnen wachsen im Garten.";
        let encoded = tokenizer(MULTILINGUAL_MODEL).encode(text, false).unwrap();
        assert!(!encoded.is_empty());

        let mut previous = (0, 0);
        for &(start, end) in encoded.get_offsets() {
            assert!(
                start <= end && end <= text.len(),
                "offset ({start}, {end}) out of bounds"
            );
            assert!(
                text.is_char_boundary(start) && text.is_char_boundary(end),
                "offset ({start}, {end}) splits a character"
            );
            assert!(
                start >= previous.0 && end >= previous.1,
                "offset ({start}, {end}) goes backwards after {previous:?}"
            );
            previous = (start, end);
        }
    }
}
