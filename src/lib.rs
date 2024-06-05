use aho_corasick::Span;
use rayon::prelude::*;
use std::str::from_utf8;

use crate::ac_matches::find_patterns_matches;
use crate::common::{merge_split_results, merge_splits, Split, SplitResults};
use crate::config::SplitterConfig;
use crate::encodings::Tokenize;

pub mod ac_matches;
pub mod common;
pub mod config;
pub mod encodings;
#[cfg(feature = "tokenizers")]
pub mod hf_tokenizer;
pub mod normalizer;
pub mod pattern_search;
mod py_binding;
pub mod splitter;
pub mod ws_tokenizer;

#[inline]
pub fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

pub fn text_split_parallel<T: Tokenize + Sync>(
    conf: &SplitterConfig<T>,
    data: &str,
) -> Vec<SplitResults> {
    let mut matches_offsets;
    let mut first_level_pattern_id = 0;

    loop {

        let match_result =
            find_patterns_matches(&conf.pattern[first_level_pattern_id], data.as_bytes());
        matches_offsets = match_result.splits;
        if match_result.matched {
            break;
        } else {
            if first_level_pattern_id + 1 >= conf.pattern.len() {
                break;
            }
            first_level_pattern_id += 1;
        }
    }

    let patterns = conf.pattern.clone();

    let splits: Vec<_> = if conf.parallel {
        matches_offsets
            .par_iter()
            .flat_map(|dt_span| {
                sub_splits(
                    conf,
                    data,
                    dt_span.clone(),
                    patterns.clone(),
                    first_level_pattern_id,
                )
            })
            .collect()
    } else {
        matches_offsets
            .iter()
            .flat_map(|dt_span| {
                sub_splits(
                    conf,
                    data,
                    dt_span.clone(),
                    patterns.clone(),
                    first_level_pattern_id,
                )
            })
            .collect()
    };

    if let Some(merge_level) = conf.merge_level {
        if splits.len() > 1 && merge_level <= first_level_pattern_id {
            return merge_split_results(&splits, conf.max_tokens);
        }
    }

    splits
}

fn sub_splits<T: Tokenize + Sync>(
    conf: &SplitterConfig<T>,
    data: &str,
    data_span: Span,
    patterns: Vec<Vec<String>>,
    pattern_id: usize,
) -> Vec<SplitResults> {
    let data_bytes = &data.as_bytes()[data_span.start..data_span.end];
    let data_str = std::str::from_utf8(&data_bytes).unwrap();

    let encoded = conf.tokenizer.encode(data_str).unwrap();

    let tokens_offsets = encoded.get_offsets();
    let tokens_words = encoded.get_word_ids();

    let tokens_span = Span {
        start: 0,
        end: encoded.len(),
    };

    if tokens_span.len() > conf.max_tokens {
        let sub_splits = text_split(
            tokens_offsets,
            tokens_words,
            tokens_span,
            data_span,
            patterns,
            pattern_id + 1,
            conf,
            data,
        );

        let mut merged_sub_splits = Vec::new();

        if let Some(merge_level) = conf.merge_level {
            if sub_splits.len() > 1 && merge_level <= pattern_id {
                let merged_child_splits = merge_splits(&sub_splits, conf.max_tokens);
                merged_sub_splits.extend(merged_child_splits);
            }
        }

        if merged_sub_splits.is_empty() {
            merged_sub_splits.extend(sub_splits);
        }

        let token_spans = merged_sub_splits
            .iter()
            .map(|split| split.tokens_span.clone())
            .collect();

        let actual_offsets = encoded.to_data_offsets(token_spans, data_span);

        let split_spans = actual_offsets
            .iter()
            .zip(merged_sub_splits.iter())
            .map(|(offset_span, split)| Split {
                tokens_span: split.tokens_span,
                data_span: Some(offset_span.clone()),
            })
            .collect();

        encoded.to_split_results(&split_spans, data)
    } else if !tokens_span.is_empty() {
        encoded.to_split_results(
            &vec![Split {
                tokens_span,
                data_span: Some(data_span),
            }],
            data,
        )
    } else {
        let _unused_res = encoded.to_split_results(
            &vec![Split {
                tokens_span: Span { start: 0, end: 0 },
                data_span: Some(data_span),
            }],
            data,
        );
        vec![]
    }
}

pub fn next_split(tokens_offsets: &[(usize, usize)], tokens_span: Span, pt_span: Span) -> Split {
    let tk_next_pos_opt = encodings::next_token_pos(tokens_offsets, tokens_span, pt_span.end);

    match tk_next_pos_opt {
        Some(tk_pos) => {
            let tk_span = span(tokens_span.start, tokens_span.start + tk_pos);
            Split {
                tokens_span: tk_span,
                data_span: None,
            }
        }
        _ => Split {
            tokens_span,
            data_span: None,
        },
    }
}

pub fn text_split<T: Tokenize + Sync>(
    tokens_offsets: &[(usize, usize)],
    tokens_words: &[Option<u32>],
    in_tokens_span: Span,
    data_span: Span,
    patterns: Vec<Vec<String>>,
    pattern_id: usize,
    conf: &SplitterConfig<T>,
    data: &str,
) -> Vec<Split> {
    let mut splits = Vec::new();
    let mut tokens_span = in_tokens_span;
    let pt_spans = if pattern_id < patterns.len() {
        let data_bytes = &data.as_bytes()[data_span.start..data_span.end];

        let mt_spans = find_patterns_matches(&patterns[pattern_id], data_bytes).splits;
        //let mt_spans_offsets = mt_spans.iter().map(|x| span(data_span.start + x.start, data_span.start + x.end)).collect();
        mt_spans
    } else {
        vec![Span {
            start: data_span.start,
            end: data_span.end,
        }]
    };

    let mut pt_spans_idx: usize = 0;

    loop {
        let split = next_split(tokens_offsets, tokens_span, pt_spans[pt_spans_idx]);

        if split.tokens_span.is_empty() {
            continue;
        }

        if split.no_tokens() > conf.max_tokens {
            if pattern_id + 1 < conf.pattern.len() {
                let child_splits = text_split(
                    tokens_offsets,
                    tokens_words,
                    split.tokens_span,
                    pt_spans[pt_spans_idx],
                    patterns.clone(),
                    pattern_id + 1,
                    conf,
                    data,
                );

                if let Some(merge_level) = conf.merge_level {
                    if child_splits.len() > 1 && merge_level <= pattern_id {
                        let merged_child_splits = merge_splits(&child_splits, conf.max_tokens);
                        splits.extend(merged_child_splits);
                    } else {
                        splits.extend(child_splits);
                    }
                } else {
                    splits.extend(child_splits);
                }
            } else {
                let sub_splits = split_tokens_len(tokens_words, split.tokens_span, conf.max_tokens);
                splits.extend(sub_splits);
            };
        } else {
            splits.push(split);
        }

        pt_spans_idx += 1;
        if pt_spans_idx < pt_spans.len() && split.tokens_span.end < in_tokens_span.end {
            tokens_span = span(split.tokens_span.end, in_tokens_span.end);
        } else {
            break;
        }
    }
    splits
}

pub fn split_tokens_len(
    tokens_words: &[Option<u32>],
    in_tokens_span: Span,
    max_tokens: usize,
) -> Vec<Split> {
    let mut splits = Vec::new();

    if in_tokens_span.len() <= max_tokens {
        splits.push(Split::new(in_tokens_span));
    } else {
        let mut split_start = in_tokens_span.start;
        let mut split_end = split_start + max_tokens;
        loop {
            if split_start >= in_tokens_span.end {
                break;
            }

            if split_end >= in_tokens_span.end {
                splits.push(Split::new(span(split_start, in_tokens_span.end)));
                break;
            }

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
                    splits.push(Split::new(span(split_start, nw_split_end)));
                    split_start = nw_split_end;
                    split_end = split_start + max_tokens;
                } else {
                    splits.push(Split::new(span(split_start, split_end)));
                    split_start = split_end;
                    split_end = split_start + max_tokens;
                }
            } else {
                splits.push(Split {
                    tokens_span: span(split_start, split_end),
                    data_span: None,
                });
                split_start = split_end;
                split_end = split_start + max_tokens;
            }
        }
    }
    splits
}
