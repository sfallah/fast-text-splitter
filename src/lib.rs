use aho_corasick::Span;
use rayon::prelude::*;

use crate::ac_matches::find_matches;
use crate::common::{Split, SplitResults};
use crate::config::SplitterConfig;
use crate::encodings::Tokenize;

pub mod ac_matches;
pub mod common;
pub mod config;
pub mod encodings;
#[cfg(feature = "tokenizers")]
pub mod hf_tokenizer;
pub mod ws_tokenizer;
mod py_binding;

#[inline]
fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

pub fn next_split(
    tokens_offsets: &[(usize, usize)],
    tokens_span: Span,
    pt_span: Span,
) -> Split {
    let tk_next_pos_opt =
        encodings::next_token_pos(tokens_offsets, tokens_span, pt_span.end);

    match tk_next_pos_opt {
        Some(tk_pos) => {
            let tk_span = span(tokens_span.start, tokens_span.start + tk_pos);
            Split {
            tokens_span: tk_span,
            data_span: None,
        }},
        _ => Split {
            tokens_span,
            data_span: None,
        },
    }
}

pub fn text_split_parallel<T: Tokenize + Sync>(
    conf: &SplitterConfig<T>,
    data: &str,
) -> Vec<SplitResults> {
    let matches_offsets = find_matches(conf.pattern[0].as_str(), data);

    let patterns = conf.pattern.clone();

    let splits = if conf.parallel {
        matches_offsets
            .par_iter()
            .flat_map(|dt_span| {
                sub_splits(&conf.tokenizer, conf, data, dt_span.clone(), patterns.clone())
            })
            .collect()
    } else {
        matches_offsets
            .iter()
            .flat_map(|dt_span| {
                sub_splits(&conf.tokenizer, conf, data, dt_span.clone(), patterns.clone())
            })
            .collect()
    };

    splits
}

fn sub_splits<T: Tokenize + Sync>(
    tokenizer: &T,
    conf: &SplitterConfig<T>,
    data: &str,
    data_span: Span,
    patterns: Vec<String>,
) -> Vec<SplitResults> {
    let encoded = tokenizer.encode(&data[data_span.start..data_span.end]).unwrap();
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
            1,
            conf,
            data,
        );

        let token_spans = sub_splits
            .iter()
            .map(|split| split.tokens_span.clone())
            .collect();

        let actual_offsets = encoded.to_data_offsets(token_spans, data_span);

        let split_spans = actual_offsets
            .iter()
            .zip(sub_splits.iter())
            .map(|(span, split)| {
                Split {
                    tokens_span: split.tokens_span,
                    data_span: Some(span.clone()),
                }
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
        vec![]
    }
}

pub fn text_split<T: Tokenize + Sync>(
    tokens_offsets: &[(usize, usize)],
    tokens_words: &[Option<u32>],
    in_tokens_span: Span,
    data_span: Span,
    patterns: Vec<String>,
    pattern_id: usize,
    conf: &SplitterConfig<T>,
    data: &str,
) -> Vec<Split> {
    let mut splits = Vec::new();
    let mut tokens_span = in_tokens_span;
    let pt_spans = find_matches(patterns[pattern_id].clone().as_str(), &data[data_span.start..data_span.end]);

    let mut pt_spans_idx: usize = 0;

    loop {
        let split = next_split(
            tokens_offsets,
            tokens_span,
            pt_spans[pt_spans_idx],
        );

        if split.no_tokens() > 0 {
            if split.no_tokens() > conf.max_tokens {
                if pattern_id + 1 < conf.max_depth {
                    let child_splits = text_split(
                        tokens_offsets,
                        tokens_words,
                        span(split.tokens_span.start, split.tokens_span.end),
                        pt_spans[pt_spans_idx],
                        patterns.clone(),
                        pattern_id + 1,
                        conf,
                        data,
                    );

                    if let Some(merge_level) = conf.merge_level {
                        if merge_level <= pattern_id + 1 {
                            let merged_child_splits = merge_splits(&child_splits, conf.max_tokens);
                            splits.extend(merged_child_splits);
                        }
                    } else {
                        splits.extend(child_splits);
                    }
                } else {
                    let sub_splits = split_tokens_len(
                        tokens_words,
                        split.tokens_span,
                        conf.max_tokens,
                    );
                    splits.extend(sub_splits);
                };
            } else {
                splits.push(split);
            }
        }

        pt_spans_idx += 1;
        if pt_spans_idx < pt_spans.len() &&
            split.tokens_span.end < in_tokens_span.end
        {
            tokens_span = span(split.tokens_span.end, in_tokens_span.end);
        } else {
            break;
        }
    }
    splits
}

pub fn merge_splits(splits: &[Split], max_tokens: usize) -> Vec<Split> {
    let mut merged_splits = Vec::new();

    let mut start_split = splits.first().unwrap();
    let mut last_split = start_split;
    let mut cur_no_tokens: usize = start_split.no_tokens();

    for (i, split) in splits.iter().enumerate().skip(1) {
        if cur_no_tokens + split.no_tokens() <= max_tokens {
            cur_no_tokens += split.no_tokens();
            last_split = split;
        } else {
            merged_splits.push(Split::new(
                span(start_split.tokens_span.start, last_split.tokens_span.end),
            ));

            start_split = split;
            last_split = split;
            cur_no_tokens = split.no_tokens();
        }
        if i == splits.len() - 1 {
            merged_splits.push(Split::new(
                span(start_split.tokens_span.start, split.tokens_span.end),
            ));
        }
    }
    merged_splits
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
                splits.push(Split::new(
                    span(split_start, in_tokens_span.end),
                ));
                break;
            }

            let split_end_word = tokens_words.get(split_end - 1).unwrap();
            let next_split_start_word = tokens_words.get(split_end).unwrap();

            if split_end_word == next_split_start_word {
                let split_end_pos = tokens_words
                    .iter()
                    .skip(split_start)
                    .take(max_tokens)
                    .rev()
                    .position(|&x| x != *split_end_word)
                    .unwrap();
                splits.push(Split::new(
                    span(split_start, split_start + max_tokens - split_end_pos),
                ));
                split_start = split_start + max_tokens - split_end_pos;
                split_end = split_start + max_tokens;
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
