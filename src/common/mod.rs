use crate::common::split::Split;
use aho_corasick::Span;

pub mod split;
mod tests;
pub mod tokens_result_lite;

pub fn chunk_encoding_splits(spans: &Vec<Split>, max_len: usize, pattern_id: usize) -> Vec<Split> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut cur_split = spans[0];
    let mut cur_tokens_len: usize = cur_split.no_tokens();
    assert!(
        cur_tokens_len <= max_len,
        "First Span: {:?} with len: {} is larger than max_len: {}",
        cur_split,
        cur_split.no_tokens(),
        max_len
    );
    if spans.len() == 1 {
        return vec![cur_split];
    }

    let mut chunked_splits = Vec::new();

    for (idx, leaf) in spans.iter().enumerate().skip(1) {
        //FIXME: not good for performance, need to remove assert in production
        assert!(
            cur_split.no_tokens() <= max_len,
            "Current Span: {:?} with len: {} is larger than max_len: {}",
            cur_split,
            cur_split.no_tokens(),
            max_len
        );

        /*
        if leaf.0.unwrap().is_empty() {
            cur_split = (cur_split.0, span(cur_split.1.start, leaf.1.end));
        } else
         */
        if cur_tokens_len + leaf.no_tokens() > max_len {
            chunked_splits.push(cur_split);
            cur_split = leaf.clone();
            cur_tokens_len = leaf.no_tokens();
        } else {
            let tokens_span = if let Some(tokens_span) = cur_split.tokens_span {
                Some(span(tokens_span.start, leaf.tokens_span.unwrap().end))
            } else {
                None
            };

            let data_span = if let Some(data_span) = cur_split.data_span {
                Some(span(data_span.start, leaf.data_span.unwrap().end))
            } else {
                None
            };
            cur_split = Split {
                tokens_span,
                data_span,
                pattern_id,
            };
            cur_tokens_len += leaf.no_tokens();
        }
        if idx == spans.len() - 1 {
            chunked_splits.push(cur_split);
        }
    }

    chunked_splits
}

#[inline]
pub fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

pub fn span_ge_offset(span: Span, offset: usize) -> Span {
    let start = span.start - offset;
    let end = span.end - offset;
    Span { start, end }
}
