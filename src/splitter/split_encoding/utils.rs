use crate::common::{span, span_ge_offset};
use aho_corasick::Span;

pub fn splits_tokens_spans(
    tokens_offsets: &[(usize, usize)],
    data_spans: &Vec<Span>,
    data_offset: usize,
    tokens_offset: usize,
) -> Vec<Span> {
    let data_spans = ne_offset_spans(&data_spans, data_offset);
    let res_spans = data_to_token_offsets(&tokens_offsets[tokens_offset..], &data_spans);
    res_spans
        .iter()
        .map(|sp| sp.offset(tokens_offset))
        .collect()
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

pub fn ne_offset_spans(spans: &Vec<Span>, offset: usize) -> Vec<Span> {
    spans.iter().map(|sp| span_ge_offset(*sp, offset)).collect()
}

#[inline]
pub fn span_contains(sp: &Span, start: usize, end: usize) -> bool {
    !sp.is_empty() && sp.start <= start && end <= sp.end
}
