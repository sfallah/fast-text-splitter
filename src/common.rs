use aho_corasick::Span;
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Split {
    pub tokens_span: Span,
    pub data_span: Option<Span>,
}

impl Split {
    pub fn new(tokens_span: Span) -> Self {
        Self {
            tokens_span,
            data_span: None,
        }
    }
}

impl fmt::Debug for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SplitOffset {{ tokens_span: {:?}, no_tokens: {}}}",
            self.tokens_span,
            self.no_tokens(),
        )
    }
}

impl Split {
    pub fn no_tokens(&self) -> usize {
        self.tokens_span.len()
    }
}


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

#[derive(PartialEq, Clone)]
pub struct SplitResults {
    pub split: Split,

    pub results: Option<TokensResults>,
    pub split_strings: String,
}

impl fmt::Debug for SplitResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SplitResults {{ split: {:?}, \n split_strings: {:?} }}",
            self.split, self.split_strings
        )
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct TokensResults {
    pub ids: Vec<u32>,
    pub type_ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub offsets: Vec<(usize, usize)>,
    pub tokens: Vec<String>,
}

impl TokensResults {
    pub fn extend(&mut self, other: TokensResults) {
        self.ids.extend(other.ids);
        self.type_ids.extend(other.type_ids);
        self.attention_mask.extend(other.attention_mask);
        self.offsets.extend(other.offsets);
        self.tokens.extend(other.tokens);
    }
}

pub struct TokensResultLite<'a> {
    pub data_span: Span,
    pub ids: Option<&'a [u32]>,
    pub offsets: Option<&'a [(usize, usize)]>,
}

impl TokensResultLite<'_> {
    pub fn no_tokens(&self) -> usize {
        if let Some(ids) = self.ids {
            ids.len()
        } else {
            if let Some(offsets) = self.offsets {
                offsets.len()
            } else {
                self.data_span.len()
            }
        }
    }
}

impl fmt::Debug for TokensResultLite<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TokensResultLite {{ data_span: {:?}, ids: {:?}, offsets: {:?} }}",
            self.data_span,
            self.no_tokens(),
            self.offsets.map_or_else(|| 0, |offsets| offsets.len())
        )
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_result_extend_test() {
        let tk_res = TokensResults::default();
        assert!(tk_res.ids.is_empty());
        assert!(tk_res.type_ids.is_empty());
        assert!(tk_res.attention_mask.is_empty());
        assert!(tk_res.offsets.is_empty());
    }

    #[test]
    fn vec_extend_test() {
        let mut empty_vec = vec![];
        let vec1 = vec![1, 2, 3];
        empty_vec.extend(vec1);
        assert_eq!(empty_vec, vec![1, 2, 3]);

        let mut vec2 = vec![4, 5, 6];
        let empty_vec2: Vec<i32> = vec![];
        vec2.extend(empty_vec2);
        assert_eq!(vec2, vec![4, 5, 6]);
    }

    #[test]
    fn tokens_result_extend_test2() {
        let mut tk_res = TokensResults::default();
        let mut tk_res2 = TokensResults::default();
        tk_res2.ids = vec![1, 2, 3];
        tk_res2.type_ids = vec![4, 5, 6];
        tk_res2.attention_mask = vec![7, 8, 9];
        tk_res2.offsets = vec![(1, 2), (3, 4), (5, 6)];
        tk_res.extend(tk_res2);
        assert_eq!(tk_res.ids, vec![1, 2, 3]);
        assert_eq!(tk_res.type_ids, vec![4, 5, 6]);
        assert_eq!(tk_res.attention_mask, vec![7, 8, 9]);
        assert_eq!(tk_res.offsets, vec![(1, 2), (3, 4), (5, 6)]);
    }

    #[test]
    fn tokens_result_default_test() {
        let tk_res = TokensResults::default();
        assert!(tk_res.ids.is_empty());
        assert!(tk_res.type_ids.is_empty());
        assert!(tk_res.attention_mask.is_empty());
        assert!(tk_res.offsets.is_empty());
    }
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
