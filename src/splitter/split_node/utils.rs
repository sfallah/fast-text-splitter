use crate::common::span;
use crate::common::tokens_result_lite::TokensResultLite;
use aho_corasick::Span;
use std::str::from_utf8;

#[derive(Debug)]
pub struct SplitResultLite {
    pub tokens: Option<Vec<u32>>,
    pub split_string: String,
}

impl SplitResultLite {
    pub fn no_tokens(&self) -> usize {
        self.tokens.as_ref().map_or(0, |tokens| tokens.len())
    }

    pub fn tokens_is_empty(&self) -> bool {
        self.tokens
            .as_ref()
            .map_or(true, |tokens| tokens.is_empty())
    }
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
                tokens: res.ids.map(|ids| ids.to_vec()),
                split_string: from_utf8(&data[res.data_span.range()]).unwrap().to_string(),
            })
            .collect();
    }

    let mut merged_results = Vec::new();
    let first_sp_res = split_ruslts.first().unwrap();

    let mut cur_data_span = first_sp_res.data_span.clone();
    let mut cur_tokens = first_sp_res.ids.map(|ids| ids.to_vec());
    let mut cur_no_tokens = first_sp_res.no_tokens();
    let mut cur_pattern_id = first_sp_res.pattern_id;

    let mut prev_tokens: Option<Vec<u32>> = None;
    let mut prev_data_span: Option<Span> = None;
    let mut has_prev = false;
    let mut prev_pattern_id: Option<usize> = None;

    for (i, split_res) in split_ruslts.iter().enumerate().skip(1) {
        if cur_pattern_id == split_res.pattern_id
            && cur_no_tokens + split_res.no_tokens() <= max_tokens
        {
            cur_tokens.as_mut().and_then(|tokens| {
                tokens.extend(split_res.ids.map_or_else(Vec::new, |ids| ids.to_vec()));
                Some(tokens)
            });
            cur_data_span = span(cur_data_span.start, split_res.data_span.end);
            cur_no_tokens += split_res.no_tokens();
        } else {
            if has_prev {
                if prev_pattern_id.clone().unwrap() == split_res.pattern_id
                    && cur_no_tokens + prev_no_tokens(&mut prev_tokens, &mut prev_data_span)
                        <= max_tokens
                {
                    prev_tokens.as_mut().and_then(|tokens| {
                        tokens.extend(cur_tokens.map_or_else(Vec::new, |ids| ids.to_vec()));
                        Some(tokens)
                    });
                    prev_data_span = Some(span(prev_data_span.unwrap().start, cur_data_span.end));
                } else {
                    merged_results.push(SplitResultLite {
                        tokens: prev_tokens.clone(),
                        split_string: from_utf8(&data[prev_data_span.unwrap().range()])
                            .unwrap()
                            .to_string(),
                    });
                    prev_tokens = cur_tokens.clone();
                    prev_data_span = Some(cur_data_span.clone());
                    prev_pattern_id = Some(cur_pattern_id);
                }
            } else {
                has_prev = true;
                prev_tokens = cur_tokens.clone();
                prev_data_span = Some(cur_data_span.clone());
                prev_pattern_id = Some(cur_pattern_id);
            }

            cur_tokens = split_res.ids.map(|ids| ids.to_vec());
            cur_data_span = split_res.data_span.clone();
            cur_no_tokens = split_res.no_tokens();
            cur_pattern_id = split_res.pattern_id;
        }

        if i == split_ruslts.len() - 1 {
            if has_prev {
                if prev_pattern_id.clone().unwrap() == split_res.pattern_id
                    && cur_no_tokens + prev_no_tokens(&mut prev_tokens, &mut prev_data_span)
                        <= max_tokens
                {
                    prev_tokens.as_mut().and_then(|tokens| {
                        tokens.extend(split_res.ids.map_or_else(Vec::new, |ids| ids.to_vec()));
                        Some(tokens)
                    });
                    merged_results.push(SplitResultLite {
                        tokens: prev_tokens.clone(),
                        split_string: from_utf8(
                            &data[prev_data_span.unwrap().start..cur_data_span.end],
                        )
                        .unwrap()
                        .to_string(),
                    });
                } else {
                    merged_results.push(SplitResultLite {
                        tokens: prev_tokens.clone(),
                        split_string: from_utf8(&data[prev_data_span.unwrap().range()])
                            .unwrap()
                            .to_string(),
                    });
                    merged_results.push(SplitResultLite {
                        tokens: cur_tokens.clone(),
                        split_string: from_utf8(&data[cur_data_span.range()]).unwrap().to_string(),
                    });
                }
            } else {
                merged_results.push(SplitResultLite {
                    tokens: cur_tokens.clone(),
                    split_string: from_utf8(&data[cur_data_span.range()]).unwrap().to_string(),
                });
            }
        }
    }
    merged_results
}

pub fn prev_no_tokens(
    prev_tokens: &mut Option<Vec<u32>>,
    prev_data_span: &mut Option<Span>,
) -> usize {
    if let Some(tokens) = prev_tokens {
        tokens.len()
    } else {
        prev_data_span.as_ref().map_or(0, |span| span.len())
    }
}
