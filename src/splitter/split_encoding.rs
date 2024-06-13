use aho_corasick::Span;
use tokenizers::parallelism::MaybeParallelRefIterator;

use crate::common::{span, span_ge_offset};
use crate::encodings::EncodingType;

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
// we need to find the range of tokens that are
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_offset() {
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let data_spans = vec![span(0, 2), span(2, 4), span(4, 5)];
        let data_offset = 0;

        let res = data_to_token_offsets(&offsets, &data_spans);
        assert_eq!(res, vec![span(0, 2), span(2, 4), span(4, 5)]);
    }

    #[test]
    fn no_tokens_data_span() {
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4)];
        let data_spans = vec![span(0, 2), span(2, 4), span(4, 5)];
        let data_offset = 0;

        let res = data_to_token_offsets(&offsets, &data_spans);
        assert_eq!(res, vec![span(0, 2), span(2, 4), span(4,4)]);
    }

    #[test]
    fn test_with_offset() {
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let data_spans = vec![span(4, 6), span(6, 8), span(8, 9)];
        let data_offset = 4;

        let data_spans = ne_offset_spans(&data_spans, data_offset);

        let res = data_to_token_offsets(&offsets, &data_spans);
        assert_eq!(res, vec![span(0, 2), span(2, 4), span(4, 5)]);
    }

    #[test]
    pub fn span_range_test() {
        let sp = span(2, 5);
        assert_eq!(sp.range(), 2..5);
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let slice = &offsets[sp.range()];
        assert_eq!(slice, &[(2, 3), (3, 4), (4, 5)]);
    }

    #[test]
    fn test_with_offset_sliced() {
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let data_spans = vec![span(6, 8), span(8, 9)];
        let data_offset: usize = 4;

        let res = splits_tokens_spans(&offsets, &data_spans, 4, 2);
        assert_eq!(res, vec![span(2, 4), span(4, 5)]);
    }
}
