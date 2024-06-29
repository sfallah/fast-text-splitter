mod tests;
mod utils;

use aho_corasick::Span;

use crate::common::{span, span_ge_offset};
use crate::encodings::EncodingType;
use crate::splitter::split_encoding::utils::splits_tokens_spans;

pub struct SplitEncoding {
    pub encoding_data_span: Span,
    pub encoding: EncodingType,
}

impl SplitEncoding {
    pub fn split_encoding_tokens_spans(
        &self,
        data_spans: &Vec<Span>,
        tokens_offset: usize,
    ) -> Vec<Span> {
        let tokens_offsets = self.encoding.get_offsets();
        let data_offset = self.encoding_data_span.start;
        splits_tokens_spans(tokens_offsets, data_spans, data_offset, tokens_offset)
    }
}

