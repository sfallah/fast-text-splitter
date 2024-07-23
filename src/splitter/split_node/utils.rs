use aho_corasick::Span;
use crate::common::span;
use crate::common::split::Split;

#[derive(Debug)]
pub struct SplitResultLite {
    pub tokens: Vec<u32>,
    pub split_string: String,
    pub data_offset: Span
}

pub fn add_splits(merged: &mut Vec<Split>, to_merge: &Vec<Split>, max_tokens: usize) {
    if to_merge.is_empty() {
        return;
    }

    if merged.is_empty() {
        merged.extend_from_slice(to_merge.as_slice());
        return;
    }

    if to_merge.len() == 1 {
        let last_merged = merged.last().unwrap().clone();
        let to_merge_split = to_merge.first().unwrap();
        if last_merged.pattern_id <= to_merge_split.pattern_id || to_merge_split.pattern_node {
            let to_merge_splits = vec![last_merged.clone(), to_merge_split.clone()];
            let merged_res = merge_splits(&to_merge_splits, max_tokens);
            merged.pop();
            merged.extend_from_slice(merged_res.as_slice());
        } else {
            merged.extend_from_slice(to_merge.as_slice());
        }
    } else {
        merged.extend_from_slice(to_merge.as_slice());
    }
}

pub fn add_splits_no_merge(merged: &mut Vec<Split>, to_merge: &Vec<Split>, max_tokens: usize) {
    if to_merge.is_empty() {
        return;
    }

    if merged.is_empty() {
        merged.extend_from_slice(to_merge.as_slice());
        return;
    }

    if to_merge.len() == 1 {
        let last_merged = merged.last().unwrap().clone();
        let to_merge_split = to_merge.first().unwrap();
        if to_merge_split.pattern_node {
            let to_merge_splits = vec![last_merged.clone(), to_merge_split.clone()];
            let merged_res = merge_splits(&to_merge_splits, max_tokens);
            merged.pop();
            merged.extend_from_slice(merged_res.as_slice());
        } else {
            merged.extend_from_slice(to_merge.as_slice());
        }
    } else {
        merged.extend_from_slice(to_merge.as_slice());
    }
}

pub fn attach_pattern_nodes(splits: &Vec<Split>, max_tokens: usize) -> Vec<Split> {
    if splits.len() <= 1 {
        return splits.to_vec();
    }
    let mut attached_splits = Vec::new();
    let mut prev_split = splits.first().unwrap().clone();
    attached_splits.push(prev_split.clone());
    for cur_split in splits.iter().skip(1) {
        if cur_split.pattern_node {
            if prev_split.no_tokens() + cur_split.no_tokens() <= max_tokens {
                let merged_split = into_one_split(&vec![prev_split.clone(), cur_split.clone()]);
                attached_splits.pop();
                attached_splits.push(merged_split.clone());
                prev_split = merged_split.clone();
                continue;
            }
        }
        attached_splits.push(cur_split.clone());
        prev_split = cur_split.clone();
    }
    attached_splits
}

pub fn merge_splits(splits: &Vec<Split>, max_tokens: usize) -> Vec<Split> {
    if splits.len() <= 1 {
        return splits.to_vec();
    }

    let mut merged_splits = Vec::new();
    let first_split = splits.first().unwrap();
    let mut cur_splits = vec![first_split.clone()];
    let mut cur_no_tokens = first_split.no_tokens();

    for (i, split_res) in splits.iter().enumerate().skip(1) {
        let mut cur_split = split_res.clone();

        if split_res.pattern_node {
            if cur_splits.len() >= 1 {
                let last_merge_split = cur_splits.pop().unwrap();
                if last_merge_split.no_tokens() + cur_split.no_tokens() <= max_tokens {
                    cur_no_tokens -= last_merge_split.no_tokens();
                    cur_split = into_one_split(&vec![last_merge_split, cur_split.clone()]);
                } else {
                    cur_splits.push(last_merge_split);
                }
            }
        }

        if cur_no_tokens + cur_split.no_tokens() <= max_tokens {
            cur_splits.push(cur_split.clone());
            cur_no_tokens += cur_split.no_tokens();
        } else {
            merged_splits.push(cur_splits.clone());
            cur_splits = vec![cur_split.clone()];
            cur_no_tokens = cur_split.no_tokens();
        }

        if i == splits.len() - 1 {
            merged_splits.push(cur_splits.clone());
        }
    }
    merged_splits
        .iter()
        .map(|splits| into_one_split(splits))
        .collect()
}

pub fn into_one_split(splits: &Vec<Split>) -> Split {
    if splits.len() == 1 {
        splits.first().unwrap().clone()
    } else {
        let first_split = splits.first().unwrap();
        let last_split = splits.last().unwrap();
        // sum tokens_no
        let sum_tokens_no: usize = splits
            .iter()
            .map(|split| split.tokens_no.unwrap_or(0))
            .sum();
        let tokens_no = if sum_tokens_no > 0 {
            Some(sum_tokens_no)
        } else {
            None
        };

        let data_span = if let Some(data_span) = first_split.data_span {
            Some(span(data_span.start, last_split.data_span.unwrap().end))
        } else {
            None
        };

        let merged_tokens_results = splits.iter().fold(Vec::new(), |mut acc, split| {
            if let Some(spl_tokens_results) = split.tokens_results.as_ref() {
                acc.extend_from_slice(spl_tokens_results.as_slice());
            }
            acc
        });

        let tokens_results = if !merged_tokens_results.is_empty() {
            Some(merged_tokens_results)
        } else {
            None
        };

        let tokenized = splits.iter().any(|split| split.tokenized);

        Split {
            pattern_id: first_split.pattern_id,
            tokens_no,
            data_span,
            pattern_node: false,
            tokens_results,
            tokenized,
        }
    }
}
