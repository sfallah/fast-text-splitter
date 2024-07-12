use aho_corasick::Span;

use crate::common::span;
use crate::common::tokens_result_lite::TokensResultLite;
use crate::hf_tokenizer::HFEncoding;
use crate::ws_tokenizer::WSEncoding;

pub trait Tokenize {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType>;
}

pub struct NoneTokenizer;

impl Tokenize for NoneTokenizer {
    fn encode(&self, _data: &str) -> anyhow::Result<EncodingType> {
        Ok(EncodingType::NoneEncoding)
    }
}

pub enum EncodingType {
    HFEncoding(HFEncoding),
    WSEncoding(WSEncoding),
    NoneEncoding,
}

impl EncodingType {
    pub fn get_offsets(&self) -> Option<&[(usize, usize)]> {
        match self {
            EncodingType::HFEncoding(enc) => Some(enc.get_offsets()),
            EncodingType::WSEncoding(enc) => Some(&*enc.offsets),
            EncodingType::NoneEncoding => None,
        }
    }

    pub fn get_word_ids(&self) -> Option<&[Option<u32>]> {
        match self {
            EncodingType::HFEncoding(enc) => Some(enc.get_word_ids()),
            EncodingType::WSEncoding(_) => None,
            EncodingType::NoneEncoding => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            EncodingType::HFEncoding(enc) => enc.len(),
            EncodingType::WSEncoding(enc) => enc.offsets.len(),
            EncodingType::NoneEncoding => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            EncodingType::HFEncoding(enc) => enc.is_empty(),
            EncodingType::WSEncoding(enc) => enc.offsets.is_empty(),
            EncodingType::NoneEncoding => true,
        }
    }

    pub fn to_data_offsets(
        &self,
        tokens_spans: Vec<Span>,
        offset: usize,
        data_span: Span,
    ) -> Vec<Span> {
        let mut res = Vec::new();

        if tokens_spans.len() <= 1 {
            res.push(data_span.clone());
            return res;
        }

        let mut start = data_span.start;

        tokens_spans
            .iter()
            .enumerate()
            .for_each(|(i, tokens_span)| {
                let end = if i < tokens_spans.len() - 1 {
                    let next_end = self.get_offsets().unwrap().get(tokens_span.end).unwrap().0;
                    next_end + offset
                } else {
                    data_span.end
                };
                res.push(span(start, end));
                start = end;
            });

        res
    }

    pub fn to_lite_results(&self, tokens_span: Option<Span>) -> TokensResultLite {
        match self {
            EncodingType::HFEncoding(hf_encoding) => hf_encoding.divide_encoding_lite(tokens_span),
            EncodingType::WSEncoding(ws_encoding) => ws_encoding.divide_encoding_lite(tokens_span),
            EncodingType::NoneEncoding => TokensResultLite::default(),
        }
    }
}

#[inline(always)]
pub fn next_token_pos(
    tokens_offsets: &[(usize, usize)],
    tokens_span: Span,
    match_start: usize,
) -> Option<usize> {
    tokens_offsets
        .iter()
        .skip(tokens_span.start)
        .take(tokens_span.len())
        .position(|&x| x.1 > match_start)
}
