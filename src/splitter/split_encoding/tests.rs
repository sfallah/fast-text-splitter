#[cfg(test)]
mod tests {
    use crate::common::span;
    use crate::splitter::split_encoding::utils::{data_to_token_offsets, ne_offset_spans, splits_tokens_spans};
    use super::*;

    #[test]
    fn test_zero_offset() {
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let data_spans = vec![span(0, 2), span(2, 4), span(4, 5)];

        let res = data_to_token_offsets(&offsets, &data_spans);
        assert_eq!(res, vec![span(0, 2), span(2, 4), span(4, 5)]);
    }

    #[test]
    fn no_tokens_data_span() {
        let offsets = vec![(0, 1), (1, 2), (2, 3), (3, 4)];
        let data_spans = vec![span(0, 2), span(2, 4), span(4, 5)];

        let res = data_to_token_offsets(&offsets, &data_spans);
        assert_eq!(res, vec![span(0, 2), span(2, 4), span(4, 4)]);
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

        let res = splits_tokens_spans(&offsets, &data_spans, 4, 2);
        assert_eq!(res, vec![span(2, 4), span(4, 5)]);
    }
}
