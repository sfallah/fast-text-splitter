use crate::common::{span, span_ne_offset};
use crate::encodings::EncodingType;
use aho_corasick::Span;

mod tests;

pub struct SplitEncoding {
    pub encoding_data_span: Span,
    pub encoding: EncodingType,
}

impl SplitEncoding {
    pub fn split_encoding_tokens_spans(
        &self,
        data_spans: &Vec<Span>,
        tokens_offset: usize,
    ) -> Option<Vec<Span>> {
        let tokens_offsets = self.encoding.get_offsets();
        let data_offset = self.encoding_data_span.start;
        splits_tokens_spans(tokens_offsets, data_spans, data_offset, tokens_offset)
    }
}

pub fn splits_tokens_spans(
    tokens_offsets: Option<&[(usize, usize)]>,
    data_spans: &Vec<Span>,
    data_offset: usize,
    tokens_offset: usize,
) -> Option<Vec<Span>> {
    if tokens_offsets.is_none() {
        return None;
    }
    // negative offset the data span according to the encoding data offset
    // encoding data offset start is the start of split that was encoded
    let data_spans = data_spans
        .iter()
        .map(|sp| span_ne_offset(sp, data_offset))
        .collect();
    let res_spans = data_to_token_offsets(&tokens_offsets.unwrap()[tokens_offset..], &data_spans);
    let res: Vec<_> = res_spans
        .iter()
        .map(|sp| sp.offset(tokens_offset))
        .collect();
    Some(res)
}

//for each split data span, we need to find the corresponding tokens span
// we need to find the range of tokens that are within the data span
pub fn data_to_token_offsets(offsets: &[(usize, usize)], data_spans: &Vec<Span>) -> Vec<Span> {
    let fold_res = data_spans.iter().fold((0usize, Vec::new()), |mut acc, sp| {
        let start = acc.0;
        if start >= offsets.len() {
            acc.1.push(span(start, start));
            (start, acc.1)
        } else {
            let mut end = offsets
                .iter()
                .skip(start)
                .take_while(|(st, nd)| span_contains(sp, *st, *nd))
                .count();
            end += start;
            acc.1.push(span(start, end));
            (end, acc.1)
        }
    });
    fold_res.1
}

#[inline]
pub fn span_contains(sp: &Span, start: usize, end: usize) -> bool {
    !sp.is_empty() && sp.start <= start && end <= sp.end
}
