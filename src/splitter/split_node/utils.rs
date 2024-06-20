use std::str::from_utf8;

use crate::common::{span, TokensResultLite};

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
                tokens: res.ids.to_vec(),
                split_string: from_utf8(&data[res.data_span.range()]).unwrap().to_string(),
            })
            .collect();
    }

    let mut merged_results = Vec::new();
    let first_sp_res = split_ruslts.first().unwrap();

    let mut cur_data_span = first_sp_res.data_span.clone();
    let mut cur_tokens = first_sp_res.ids.to_vec();

    for (i, split_res) in split_ruslts.iter().enumerate().skip(1) {
        if cur_tokens.len() + split_res.no_tokens() <= max_tokens {
            cur_tokens.extend(split_res.ids.to_vec());
            cur_data_span = span(cur_data_span.start, split_res.data_span.end);
        } else {
            merged_results.push(SplitResultLite {
                tokens: cur_tokens.clone(),
                split_string: from_utf8(&data[cur_data_span.range()]).unwrap().to_string(),
            });
            cur_tokens = split_res.ids.to_vec();
            cur_data_span = split_res.data_span.clone();
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
