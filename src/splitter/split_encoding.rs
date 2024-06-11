use aho_corasick::Span;

use crate::common::{span, span_min_offset};
use crate::encodings::EncodingType;

pub struct SplitEncoding {
    pub encoding_data_span: Span,
    pub encoding: EncodingType,
}

impl SplitEncoding {
    pub fn split_tokens_span(&self, split_data_span: Span, tokens_boundary: Span) -> Option<Span> {
        let split_data_span_offseted =
            span_min_offset(split_data_span, self.encoding_data_span.start);

        let idx = self
            .encoding
            .get_offsets()
            .iter()
            .skip(tokens_boundary.start)
            .take(tokens_boundary.len())
            .take_while(|(_, end)| split_data_span_offseted.contains(*end))
            .count();

        let pos = Some(span(tokens_boundary.start, tokens_boundary.start + idx));

        let _debug = pos.clone();

        pos
    }
}
