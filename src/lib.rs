extern crate num_cpus;

use aho_corasick::Span;
use rayon::prelude::*;

use crate::ac_matches::{find_patterns_matches, merge_partition_spans};
use crate::common::{Split, SplitResults, TokensResults};
use crate::config::SplitterConfig;
use crate::encodings::Tokenize;
use crate::normalizer::TextNormalizer;

pub mod ac_matches;
pub mod common;
pub mod config;
pub mod encodings;
#[cfg(feature = "tokenizers")]
pub mod hf_tokenizer;
pub mod normalizer;
mod py_binding;
pub mod ws_tokenizer;

#[inline]
fn span(start: usize, end: usize) -> Span {
    Span { start, end }
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

pub fn text_split_parallel<T: Tokenize + Sync>(
    conf: &SplitterConfig<T>,
    in_data: &str,
    text_normalize: Option<bool>,
) -> Vec<SplitResults> {
    let mut first_level_pattern_id = 0;
    let mut matches_offsets = vec![];
    let normalizer =
        TextNormalizer::new(text_normalize.unwrap_or(false), false, Some(false), false);
    let binding = normalizer
        .normalize(&in_data.clone().to_string())
        .unwrap()
        .clone();
    let data = binding.as_str();
    loop {
        let match_result =
            find_patterns_matches(&conf.pattern[first_level_pattern_id], data.as_bytes());
        matches_offsets = match_result.splits;
        if matches_offsets.len() > 1 || match_result.matched {
            break;
        } else {
            if first_level_pattern_id + 1 < conf.pattern.len() {
                first_level_pattern_id += 1;
            } else {
                break;
            }
        }
    }

    let no_cpus = num_cpus::get();

    let patterns = conf.pattern.clone();

    let partition_spans =
        merge_partition_spans(&matches_offsets, &matches_offsets.len() / no_cpus + 1);
    //let partition_spans = &matches_offsets.clone();

    let total_spans_len = partition_spans.iter().map(|span| span.len()).sum::<usize>();
    assert_eq!(total_spans_len, data.len());

    let splits: Vec<_> = if conf.parallel {
        partition_spans
            .par_iter()
            .flat_map(|dt_span| {
                sub_splits(
                    &conf.tokenizer,
                    conf,
                    data,
                    dt_span.clone(),
                    patterns.clone(),
                    first_level_pattern_id + 1,
                )
            })
            .collect()
    } else {
        partition_spans
            .iter()
            .flat_map(|dt_span| {
                sub_splits(
                    &conf.tokenizer,
                    conf,
                    data,
                    dt_span.clone(),
                    patterns.clone(),
                    first_level_pattern_id + 1,
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
    tokenizer: &T,
    conf: &SplitterConfig<T>,
    data: &str,
    data_span: Span,
    patterns: Vec<Vec<String>>,
    pattern_id: usize,
) -> Vec<SplitResults> {
    let data_bytes = &data.as_bytes()[data_span.start..data_span.end];
    let data_str = std::str::from_utf8(&data_bytes).unwrap();

    let encoded = tokenizer.encode(data_str).unwrap();

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

        let mut merged_splits = Vec::new();

        if let Some(merge_level) = conf.merge_level {
            if sub_splits.len() > 1 && merge_level <= pattern_id {
                let merged_child_splits = merge_splits(&sub_splits, conf.max_tokens);
                merged_splits.extend(merged_child_splits);
            } else {
                merged_splits.extend(&sub_splits);
            }
        } else {
            merged_splits.extend(&sub_splits);
        }

        let token_spans = merged_splits
            .iter()
            .map(|split| split.tokens_span.clone())
            .collect();

        let actual_offsets = encoded.to_data_offsets(token_spans, data_span);

        let split_spans = actual_offsets
            .iter()
            .zip(merged_splits.iter())
            .map(|(span, split)| Split {
                tokens_span: split.tokens_span,
                data_span: Some(span.clone()),
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
        encoded.to_split_results(&vec![
            Split {
                tokens_span: Span {
                    start: 0,
                    end: 0,
                },
                data_span: Some(data_span),
            },
        ], data)
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
    let pt_spans = if pattern_id <= patterns.len() - 1 {
        find_patterns_matches(
            &patterns[pattern_id],
            &data.as_bytes()[data_span.start..data_span.end],
        )
            .splits
    } else {
        vec![Span {
            start: data_span.start,
            end: data_span.end,
        }]
    };

    let mut pt_spans_idx: usize = 0;

    loop {
        let split = next_split(tokens_offsets, tokens_span, pt_spans[pt_spans_idx]);

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
                let sub_splits =
                    split_tokens_len(tokens_words, split.tokens_span, conf.max_tokens);
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

pub fn merge_splits(splits: &[Split], max_tokens: usize) -> Vec<Split> {
    if splits.len() <= 1 {
        return splits.to_vec();
    }
    let mut merged_splits = Vec::new();

    let mut start_split = splits.first().unwrap();
    let mut last_split = start_split;

    let mut cur_no_tokens: usize = start_split.no_tokens();

    for (i, split) in splits.iter().enumerate().skip(1) {
        if cur_no_tokens + split.no_tokens() <= max_tokens {
            cur_no_tokens += split.no_tokens();
            last_split = split;
        } else {
            merged_splits.push(Split::new(span(
                start_split.tokens_span.start,
                last_split.tokens_span.end,
            )));

            start_split = split;
            last_split = start_split;
            cur_no_tokens = split.no_tokens();
        }
        if i == splits.len() - 1 {
            merged_splits.push(Split::new(span(
                start_split.tokens_span.start,
                split.tokens_span.end,
            )));
        }
    }
    merged_splits
}

pub fn merge_split_results(
    split_ruslts: &Vec<SplitResults>,
    max_tokens: usize,
) -> Vec<SplitResults> {
    if split_ruslts.len() <= 1 {
        return split_ruslts.to_vec();
    }
    let mut merged_results = Vec::new();
    let first_sp_res = split_ruslts.first().unwrap();

    let mut split_no_tokens = first_sp_res.split.no_tokens();
    let mut split_strings = first_sp_res.split_strings.clone();
    let mut split_tk_start: usize = 0;

    let mut cur_tk_end: usize = first_sp_res.split.no_tokens();

    let mut split_tk_res = first_sp_res
        .results
        .clone()
        .unwrap_or(TokensResults::default());

    for (i, split_res) in split_ruslts.iter().enumerate().skip(1) {
        if split_no_tokens + split_res.split.no_tokens() <= max_tokens {
            split_no_tokens += split_res.split.no_tokens();
            split_strings.push_str(split_res.split_strings.as_str());
            split_tk_res.extend(
                split_res
                    .results
                    .clone()
                    .unwrap_or(TokensResults::default()),
            );
            cur_tk_end += split_res.split.no_tokens();
        } else {
            merged_results.push(SplitResults {
                split: Split::new(span(split_tk_start, cur_tk_end)),
                results: Some(split_tk_res.clone()),
                split_strings: split_strings.clone(),
            });

            split_no_tokens = split_res.split.no_tokens();
            split_strings = split_res.split_strings.clone();
            split_tk_res = split_res
                .results
                .clone()
                .unwrap_or(TokensResults::default());
            split_tk_start = cur_tk_end;
            cur_tk_end += split_res.split.no_tokens();
        }

        if i == split_ruslts.len() - 1 {
            merged_results.push(SplitResults {
                split: Split::new(span(split_tk_start, cur_tk_end)),
                results: Some(split_tk_res.clone()),
                split_strings: split_strings.clone(),
            });
        }
    }
    merged_results
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
                    .take(max_tokens)
                    .rev()
                    .position(|&x| x != *split_end_word);

                if let Some(split_end_pos) = split_end_pos_opt {
                    splits.push(Split::new(span(
                        split_start,
                        split_start + max_tokens - split_end_pos,
                    )));
                    split_start = split_start + max_tokens - split_end_pos;
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
