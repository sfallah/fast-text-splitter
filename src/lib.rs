use aho_corasick::{Match, Span};
use rayon::prelude::*;

use crate::ac_matches::{match_offsets, matches_spans};
use crate::common::{Split, SplitResults};
use crate::config::SplitterConfig;
use crate::encodings::tokens_data_offsets;
use crate::encodings::{EncodingType, Tokenize};

pub mod ac_matches;
pub mod common;
pub mod config;
pub mod encodings;
#[cfg(feature = "tokenizers")]
pub mod hf_tokenizer;
pub mod ws_tokenizer;

#[inline]
fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

pub fn next_split(
    tokens_offsets: &[(usize, usize)],
    tokens_span: Span,
    matches: &[Match],
    matches_span: Span,
    pattern_id: usize,
) -> Split {
    let mt_next_pos_opt = ac_matches::next_match_pos(matches, matches_span, pattern_id);

    match mt_next_pos_opt {
        Some(mt_next_pos) => {
            let mat = matches.get(matches_span.start + mt_next_pos).unwrap();

            let tk_next_pos_opt =
                encodings::next_token_pos(tokens_offsets, tokens_span, mat.start());

            match tk_next_pos_opt {
                Some(tk_pos) => Split::new(
                    span(matches_span.start, matches_span.start + mt_next_pos + 1),
                    span(tokens_span.start, tokens_span.start + tk_pos),
                ),
                _ => Split::new(
                    span(matches_span.start, matches_span.start + mt_next_pos + 1),
                    tokens_span,
                ),
            }
        }
        _ => Split::new(matches_span, tokens_span),
    }
}

pub fn text_split_parallel<T: Tokenize + Sync>(
    conf: &SplitterConfig<T>,
    data: &str,
) -> Vec<SplitResults> {
    let matches: Vec<_> = conf.aho_corasick.find_iter(data).collect();
    let matches_spans = matches_spans(&matches, 0);

    let splits = if conf.parallel {
        matches_spans
            .par_iter()
            .flat_map(|matches_span| {
                sub_splits(&conf.tokenizer, conf, data, &matches, matches_span)
            })
            .collect()
    } else {
        matches_spans
            .iter()
            .flat_map(|matches_span| {
                sub_splits(&conf.tokenizer, conf, data, &matches, matches_span)
            })
            .collect()
    };

    splits
}

fn sub_splits<T: Tokenize + Sync>(
    tokenizer: &T,
    conf: &SplitterConfig<T>,
    data: &str,
    matches: &[Match],
    matches_span: &Span,
) -> Vec<SplitResults> {
    let (start, end) = match_offsets(matches, matches_span, data.len()).unwrap();
    let encoded = tokenizer.encode(&data[start..end]).unwrap();
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
            matches,
            *matches_span,
            1,
            conf,
        );
        let split_spans: Vec<Split> = sub_splits
            .iter()
            .map(|split| {
                let (tk_start, tk_end) =
                    tokens_data_offsets(&tokens_offsets, split.tokens_span, (start, end)).unwrap();
                Split {
                    tokens_span: split.tokens_span,
                    matches_span: *matches_span,
                    data_span: Some(span(tk_start, tk_end)),
                }
            })
            .collect();
        encoded.to_split_results(&split_spans, data)
    } else if !tokens_span.is_empty() {
        encoded.to_split_results(
            &vec![Split {
                tokens_span,
                matches_span: *matches_span,
                data_span: Some(span(start, end)),
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
    matches: &[Match],
    in_matches_span: Span,
    pattern_id: usize,
    conf: &SplitterConfig<T>,
) -> Vec<Split> {
    let mut splits = Vec::new();
    let mut tokens_span = in_tokens_span;
    let mut matches_span = in_matches_span;

    loop {
        let split = next_split(
            tokens_offsets,
            tokens_span,
            matches,
            matches_span,
            pattern_id,
        );

        if split.matches_span.end <= in_matches_span.end && split.no_tokens() > 0 {
            if split.no_tokens() > conf.max_tokens {
                if pattern_id + 1 < conf.max_depth {
                    let child_splits = text_split(
                        tokens_offsets,
                        tokens_words,
                        span(split.tokens_span.start, split.tokens_span.end),
                        matches,
                        span(split.matches_span.start, split.matches_span.end),
                        pattern_id + 1,
                        conf,
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
                        split.matches_span,
                        conf.max_tokens,
                    );
                    splits.extend(sub_splits);
                };
            } else {
                splits.push(split);
            }
        }

        if split.matches_span.end < in_matches_span.end
            || split.tokens_span.end < in_tokens_span.end
        {
            tokens_span = span(split.tokens_span.end, in_tokens_span.end);
            matches_span = span(split.matches_span.end, in_matches_span.end);
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
                span(start_split.matches_span.start, last_split.matches_span.end),
                span(start_split.tokens_span.start, last_split.tokens_span.end),
            ));

            start_split = split;
            last_split = split;
            cur_no_tokens = split.no_tokens();
        }
        if i == splits.len() - 1 {
            merged_splits.push(Split::new(
                span(start_split.matches_span.start, split.matches_span.end),
                span(start_split.tokens_span.start, split.tokens_span.end),
            ));
        }
    }
    merged_splits
}

pub fn split_tokens_len(
    tokens_words: &[Option<u32>],
    in_tokens_span: Span,
    in_matches_span: Span,
    max_tokens: usize,
) -> Vec<Split> {
    let mut splits = Vec::new();

    if in_tokens_span.len() <= max_tokens {
        splits.push(Split::new(in_matches_span, in_tokens_span));
    } else {
        let mut split_start = in_tokens_span.start;
        let mut split_end = split_start + max_tokens;
        loop {
            if split_start >= in_tokens_span.end {
                break;
            }

            if split_end >= in_tokens_span.end {
                splits.push(Split::new(
                    in_matches_span,
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
                    in_matches_span,
                    span(split_start, split_start + max_tokens - split_end_pos - 1),
                ));
                split_start = split_start + max_tokens - split_end_pos;
                split_end = split_start + max_tokens;
            } else {
                splits.push(Split {
                    tokens_span: span(split_start, split_end),
                    matches_span: in_matches_span,
                    data_span: None,
                });
                split_start = split_end;
                split_end = split_start + max_tokens;
            }
        }
    }
    splits
}
