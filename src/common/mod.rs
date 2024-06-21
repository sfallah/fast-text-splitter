use aho_corasick::Span;
use crate::common::split::Split;
use crate::common::split_result::SplitResults;
use crate::common::tokens_result::TokensResults;

pub mod split;
pub mod split_result;
pub mod tokens_result;
pub mod tokens_result_lite;
mod tests;

pub fn chunk_encoding_splits(spans: &Vec<(Option<Span>, Span)>, max_len: usize) -> Vec<(Option<Span>, Span)> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut cur_split = spans[0];
    let mut cur_tokens_len: usize = cur_split.0.unwrap().len();
    assert!(
        cur_tokens_len <= max_len,
        "First Span: {:?} with len: {} is larger than max_len: {}",
        cur_split.0,
        cur_split.0.unwrap().len(),
        max_len
    );
    if spans.len() == 1 {
        return vec![cur_split];
    }

    let mut chunked_splits = Vec::new();

    for (idx, leaf) in spans.iter().enumerate().skip(1) {
        assert!(
            cur_split.0.unwrap().len() <= max_len,
            "Current Span: {:?} with len: {} is larger than max_len: {}",
            cur_split,
            cur_split.0.unwrap().len(),
            max_len
        );

        if leaf.0.unwrap().is_empty() {
            cur_split = (cur_split.0, span(cur_split.1.start, leaf.1.end));
        } else if cur_tokens_len + leaf.0.unwrap().len() > max_len {
            chunked_splits.push(cur_split);
            cur_split = leaf.clone();
            cur_tokens_len = leaf.0.unwrap().len();
        } else {
            cur_split = (
                Some(span(cur_split.0.unwrap().start, leaf.0.unwrap().end)),
                span(cur_split.1.start, leaf.1.end),
            );
            cur_tokens_len += leaf.0.unwrap().len();
        }
        if idx == spans.len() - 1 {
            chunked_splits.push(cur_split);
        }
    }

    chunked_splits
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

#[inline]
pub fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

pub fn span_ge_offset(span: Span, offset: usize) -> Span {
    let start = span.start - offset;
    let end = span.end - offset;
    Span { start, end }
}