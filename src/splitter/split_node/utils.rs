use std::str::from_utf8;

use crate::common::span;
use crate::common::tokens_result_lite::TokensResultLite;

#[derive(Debug)]
pub struct SplitResultLite {
    pub tokens: Vec<u32>,
    pub split_string: String,
}

pub fn merge_split_result_lite(
    split_ruslts: &Vec<TokensResultLite>,
    max_tokens: usize,
    data: &[u8],
) -> Vec<SplitResultLite> {
    if split_ruslts.len() <= 1 {
        return split_ruslts
            .iter()
            .map(|res| SplitResultLite {
                tokens: res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec()),
                split_string: from_utf8(&data[res.data_span.range()]).unwrap().to_string(),
            })
            .collect();
    }

    let mut merged_results = Vec::new();
    let first_sp_res = split_ruslts.first().unwrap();

    let mut cur_data_span = first_sp_res.data_span.clone();
    let mut cur_tokens = first_sp_res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec());
    let mut cur_no_tokens = first_sp_res.no_tokens();
    let mut cur_pattern_id = first_sp_res.pattern_id;

    for (i, split_res) in split_ruslts.iter().enumerate().skip(1) {
        if cur_pattern_id <= split_res.pattern_id && cur_no_tokens + split_res.no_tokens() <= max_tokens {
            cur_tokens.extend(split_res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec()));
            cur_data_span = span(cur_data_span.start, split_res.data_span.end);
            cur_no_tokens += split_res.no_tokens();
        } else {
            merged_results.push(SplitResultLite {
                tokens: cur_tokens.clone(),
                split_string: from_utf8(&data[cur_data_span.range()]).unwrap().to_string(),
            });
            cur_tokens = split_res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec());
            cur_data_span = split_res.data_span.clone();
            cur_no_tokens = split_res.no_tokens();
            cur_pattern_id = split_res.pattern_id;
        }

        if i == split_ruslts.len() - 1 {
            merged_results.push(SplitResultLite {
                tokens: cur_tokens.clone(),
                split_string: from_utf8(&data[cur_data_span.range()]).unwrap().to_string(),
            });
        }
    }
    merged_results
}

pub fn merge_split_tokens_result_lite(
    split_ruslts: &Vec<TokensResultLite>,
    max_tokens: usize,
) -> Vec<TokensResultLite> {
    if split_ruslts.len() <= 1 {
        return split_ruslts.to_vec()
    }

    let mut merged_results = Vec::new();
    let first_sp_res = split_ruslts.first().unwrap();

    let mut cur_data_span = first_sp_res.data_span.clone();
    let mut cur_tokens = first_sp_res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec());
    let mut cur_no_tokens = first_sp_res.no_tokens();
    let mut cur_offset = first_sp_res.offsets.as_ref().map_or_else(Vec::new, |offsets| offsets.to_vec());
    let mut cur_pattern_id = first_sp_res.pattern_id;

    for (i, split_res) in split_ruslts.iter().enumerate().skip(1) {
        if cur_pattern_id <= split_res.pattern_id && cur_no_tokens + split_res.no_tokens() <= max_tokens {
            cur_tokens.extend(split_res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec()));
            cur_offset.extend(split_res.offsets.as_ref().map_or_else(Vec::new, |offsets| offsets.to_vec()));
            cur_data_span = span(cur_data_span.start, split_res.data_span.end);
            cur_no_tokens += split_res.no_tokens();
        } else {
            merged_results.push(TokensResultLite {
                data_span: cur_data_span.clone(),
                ids: if cur_tokens.is_empty() { None } else { Some(cur_tokens.clone()) },
                offsets: if cur_offset.is_empty() { None } else { Some(cur_offset.clone()) },
                pattern_id: cur_pattern_id,
            });
            cur_data_span = split_res.data_span.clone();
            cur_tokens = split_res.ids.as_ref().map_or_else(Vec::new, |ids| ids.to_vec());
            cur_offset = split_res.offsets.as_ref().map_or_else(Vec::new, |offsets| offsets.to_vec());
            cur_no_tokens = split_res.no_tokens();
            cur_pattern_id = split_res.pattern_id;
        }

        if i == split_ruslts.len() - 1 {
            merged_results.push(TokensResultLite {
                data_span: cur_data_span.clone(),
                ids: if cur_tokens.is_empty() { None } else { Some(cur_tokens.clone()) },
                offsets: if cur_offset.is_empty() { None } else { Some(cur_offset.clone()) },
                pattern_id: cur_pattern_id,
            });
        }
    }
    merged_results
}
