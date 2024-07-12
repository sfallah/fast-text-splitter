use aho_corasick::Span;

use crate::common::span;
use crate::splitter::split_node::SplitNode;

pub fn split_data_len<'a>(
    pattern_id: usize,
    split_span: Span,
    max_len_value: usize,
) -> Vec<SplitNode> {
    if max_len_value == 0 {
        return vec![SplitNode::new(
            pattern_id,
            split_span,
            Vec::new(),
            None,
            None,
            false,
        )];
    }
    let mut result = Vec::new();
    let mut start = split_span.start;
    let end = split_span.end;
    while start < end {
        let chunk_end = if end - start > max_len_value {
            start + max_len_value
        } else {
            end
        };
        let chunk_span = Span::from(start..chunk_end);
        result.push(SplitNode::new(
            pattern_id,
            chunk_span,
            Vec::new(),
            None,
            None,
            false,
        ));
        start = chunk_end;
    }
    result
}

pub fn chunk_tokens_len(
    tokens_words: Option<&[Option<u32>]>,
    in_tokens_span: Span,
    max_tokens: usize,
) -> Vec<Span> {
    let mut splits = Vec::new();

    if in_tokens_span.len() <= max_tokens {
        splits.push(in_tokens_span);
    } else {
        let mut split_start = in_tokens_span.start;
        let mut split_end = split_start + max_tokens;
        loop {
            if split_start >= in_tokens_span.end {
                break;
            }

            if split_end >= in_tokens_span.end {
                splits.push(span(split_start, in_tokens_span.end));
                break;
            }

            if let Some(tokens_words) = tokens_words {
                let split_end_word = tokens_words.get(split_end - 1).unwrap();
                let next_split_start_word = tokens_words.get(split_end).unwrap();

                if split_end_word == next_split_start_word {
                    let split_end_pos_opt = tokens_words
                        .iter()
                        .skip(split_start)
                        .take(split_end - split_start)
                        .rev()
                        .position(|&x| x != *split_end_word);

                    if let Some(split_end_pos) = split_end_pos_opt {
                        let nw_split_end = split_end - split_end_pos;
                        splits.push(span(split_start, nw_split_end));
                        split_start = nw_split_end;
                        split_end = split_start + max_tokens;
                    } else {
                        splits.push(span(split_start, split_end));
                        split_start = split_end;
                        split_end = split_start + max_tokens;
                    }
                } else {
                    splits.push(span(split_start, split_end));
                    split_start = split_end;
                    split_end = split_start + max_tokens;
                }
            } else {
                splits.push(span(split_start, split_end));
                split_start = split_end;
                split_end = split_start + max_tokens;
            }
        }
    }
    splits
}
