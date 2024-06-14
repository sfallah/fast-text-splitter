use std::str::from_utf8;
use std::sync::Arc;

use aho_corasick::Span;

use crate::common::span;
use crate::encodings::Tokenize;
use crate::pattern_search::pattern_searcher::PatternSearcher;
use crate::splitter::split_encoding::SplitEncoding;

//FIXME: need to be moved to Splitter
pub struct SplitterConfig<'a, T: Tokenize + Sync> {
    pub data: &'a [u8],
    pub patterns: &'a Vec<Vec<&'a str>>,
    pub searchers: &'a Vec<PatternSearcher<'a>>,
    pub tokenizer: Option<&'a T>,
    pub max_len: Option<usize>,
}

impl<'a, T: Tokenize + Sync> SplitterConfig<'a, T> {
    pub fn encode_search_split(&self, data_span: &Span) -> Option<Arc<SplitEncoding>> {
        if let Some(tokenizer) = self.tokenizer {
            let data_str = from_utf8(&self.data[*data_span]).unwrap();
            let encoding = tokenizer.encode(data_str).unwrap();
            Some(Arc::new(SplitEncoding {
                encoding_data_span: data_span.clone(),
                encoding,
            }))
        } else {
            None
        }
    }

    //FIXME: need to be moved to Splitter
    pub fn get_split_encoding(
        &self,
        split_span: &Span,
        split_encoding: &Option<Arc<SplitEncoding>>,
        split_tokens_span: &Option<Span>,
    ) -> (Option<Arc<SplitEncoding>>, Option<Span>) {
        let new_split_encoding = split_encoding
            .clone()
            .or_else(|| self.encode_search_split(&split_span));

        let new_split_tokens_span = split_tokens_span.clone().or_else(|| {
            new_split_encoding
                .as_ref()
                .and_then(|encoding| Some(span(0, encoding.encoding.len())))
        });

        (new_split_encoding, new_split_tokens_span)
    }
}
