#[cfg(feature = "tokenizers")]
use crate::common::TokensResults;

#[cfg(feature = "tokenizers")]
use tokenizers::Encoding;
#[cfg(feature = "tokenizers")]
use crate::hf_tokenizer::divide_encoding;

use crate::{Split, SplitResults};
use aho_corasick::Span;

use crate::ws_tokenizer::WSEncoding;

pub trait Tokenize {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType>;
}

pub enum EncodingType {
    #[cfg(feature = "tokenizers")]
    HFEncoding(Encoding),
    WSEncoding(WSEncoding),
}

impl EncodingType {
    pub fn get_offsets(&self) -> &[(usize, usize)] {
        match self {
            #[cfg(feature = "tokenizers")]
            EncodingType::HFEncoding(enc) => enc.get_offsets(),
            EncodingType::WSEncoding(enc) => &*enc.offsets,
        }
    }

    pub fn get_word_ids(&self) -> &[Option<u32>] {
        match self {
            #[cfg(feature = "tokenizers")]
            EncodingType::HFEncoding(enc) => enc.get_word_ids(),
            EncodingType::WSEncoding(enc) => &*enc.word_ids,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            #[cfg(feature = "tokenizers")]
            EncodingType::HFEncoding(enc) => enc.len(),
            EncodingType::WSEncoding(enc) => enc.offsets.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            #[cfg(feature = "tokenizers")]
            EncodingType::HFEncoding(enc) => enc.is_empty(),
            EncodingType::WSEncoding(enc) => enc.offsets.is_empty(),
        }
    }

    pub fn to_data_offsets(&self, tokens_spans: Vec<Span>, data_span: Span) -> Vec<Span> {
        let mut res = Vec::new();
        let mut start = data_span.start;

        if tokens_spans.len() <= 1 {
            res.push(data_span.clone());
            return res;
        }

        tokens_spans.iter().skip(1).for_each(|tokens_span| {
            let tokens_offsets = self.get_offsets();
            let tokens_span = tokens_span.clone();
            let next_end = tokens_offsets.get(tokens_span.start).unwrap().0;
            let span = Span {
                start,
                end: data_span.start + next_end,
            };
            res.push(span);
            start = data_span.start + next_end;
        });

        res.push(Span {
                start,
                end: data_span.end,
            });


        res
    }

    pub fn to_split_results(&self, splits: &Vec<Split>, data: &str) -> Vec<SplitResults> {
        #[cfg(feature = "tokenizers")]
        let encodings: Vec<TokensResults> = match self {
            #[cfg(feature = "tokenizers")]
            EncodingType::HFEncoding(enc) => divide_encoding(enc, splits),
            EncodingType::WSEncoding(_) => vec![],
        };

        let split_strings: Vec<_> = splits
            .iter()
            .map(|split| {
                let data_span = split.data_span.unwrap();
                data[data_span.start..data_span.end].to_string()
            })
            .collect();
        let res: Vec<_> = splits
            .iter()
            .enumerate()
            .map(|(i, split)| SplitResults {
                splits: split.clone(),
                #[cfg(feature = "tokenizers")]
                results: if encodings.is_empty() {
                    None
                } else {
                    Some(encodings.get(i).unwrap().clone())
                },
                split_strings: split_strings.get(i).unwrap().clone(),
            })
            .collect();
        res
    }
}
#[inline]
pub fn tokens_data_offsets(
    tokens_offsets: &[(usize, usize)],
    tokens_span: Span,
    match_offsets: Span,
) -> Span {
    let start = if tokens_span.start > 0 {
        match_offsets.start + tokens_offsets.get(tokens_span.start).unwrap().0
    } else {
        match_offsets.start
    };

    let end = if tokens_span.end - 1 < tokens_offsets.len() {
        match_offsets.start + tokens_offsets.get(tokens_span.end - 1).unwrap().1
    } else {
        match_offsets.end
    };
    Span{start, end}
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
