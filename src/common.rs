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

pub fn chunk_spans(spans: &Vec<Span>, max_len: usize) -> Vec<Span> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut cur_split = spans[0];
    let mut cur_len: usize = cur_split.len();
    assert!(
        cur_len <= max_len,
        "First Span: {:?} with len: {} is larger than max_len: {}",
        cur_split,
        cur_split.len(),
        max_len
    );
    if spans.len() == 1 {
        return vec![cur_split];
    }

    let mut chunked_splits = Vec::new();

    for (idx, leaf) in spans.iter().enumerate().skip(1) {
        assert!(
            cur_split.len() <= max_len,
            "Current Span: {:?} with len: {} is larger than max_len: {}",
            cur_split,
            cur_split.len(),
            max_len
        );

        assert_eq!(
            cur_split.end, leaf.start,
            "Next Span: {:?} doesn't continue the Current Span: {:?}",
            leaf, cur_split
        );
        if cur_len + leaf.len() > max_len {
            chunked_splits.push(cur_split);
            cur_split = leaf.clone();
            cur_len = leaf.len();
        } else {
            cur_split = span(cur_split.start, leaf.end);
            cur_len += leaf.len();
        }
        if idx == spans.len() - 1 {
            chunked_splits.push(cur_split);
        }
    }

    chunked_splits
}

pub fn chunk_encoding_splits(spans: &Vec<(Span, Span)>, max_len: usize) -> Vec<(Span, Span)> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut cur_split = spans[0];
    let mut cur_tokens_len: usize = cur_split.0.len();
    assert!(
        cur_tokens_len <= max_len,
        "First Span: {:?} with len: {} is larger than max_len: {}",
        cur_split.0,
        cur_split.0.len(),
        max_len
    );
    if spans.len() == 1 {
        return vec![cur_split];
    }

    let mut chunked_splits = Vec::new();

    for (idx, leaf) in spans.iter().enumerate().skip(1) {
        assert!(
            cur_split.0.len() <= max_len,
            "Current Span: {:?} with len: {} is larger than max_len: {}",
            cur_split,
            cur_split.0.len(),
            max_len
        );

        if leaf.0.is_empty() {
            cur_split = (cur_split.0, span(cur_split.1.start, leaf.1.end));
        } else if cur_tokens_len + leaf.0.len() > max_len {
            chunked_splits.push(cur_split);
            cur_split = leaf.clone();
            cur_tokens_len = leaf.0.len();
        } else {
            cur_split = (
                span(cur_split.0.start, leaf.0.end),
                span(cur_split.1.start, leaf.1.end),
            );
            cur_tokens_len += leaf.0.len();
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
                0
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
    fn merge_splits_test() {
        let splits = vec![
            Split::new(span(0, 16)),
            Split::new(span(16, 24)),
            Split::new(span(24, 36)),
        ];
        let max_tokens = 20;
        let merged_splits = merge_splits(&splits, max_tokens);

        for split in &merged_splits {
            println!("{:?}", split);
        }

        assert_eq!(merged_splits.len(), 2);
        assert_eq!(merged_splits[0].tokens_span, span(0, 16));
        assert_eq!(merged_splits[1].tokens_span, span(16, 36));

        let splits2 = vec![Split::new(span(0, 16)), Split::new(span(16, 24))];
        let merged_splits2 = merge_splits(&splits2, max_tokens);
        assert_eq!(merged_splits2.len(), 2);
        assert_eq!(merged_splits2[0].tokens_span, span(0, 16));
        assert_eq!(merged_splits2[1].tokens_span, span(16, 24));

        // Test when splits is empty
        let splits3: Vec<Split> = vec![];
        let merged_splits3 = merge_splits(&splits3, max_tokens);
        assert_eq!(merged_splits3.len(), 0);

        // Test when splits has only one element
        let splits4 = vec![Split::new(span(0, 16))];
        let merged_splits4 = merge_splits(&splits4, max_tokens);
        assert_eq!(merged_splits4.len(), 1);
        assert_eq!(merged_splits4[0].tokens_span, span(0, 16));

        // test split with empty tokens
        let splits5 = vec![
            Split::new(span(0, 0)),
            Split::new(span(0, 16)),
            Split::new(span(16, 16)),
            Split::new(span(16, 24)),
            Split::new(span(24, 36)),
            Split::new(span(36, 36)),
        ];
        let merged_splits5 = merge_splits(&splits5, max_tokens);
        assert_eq!(merged_splits5.len(), 2);
        assert_eq!(merged_splits5[0].tokens_span, span(0, 16));
        assert_eq!(merged_splits5[1].tokens_span, span(16, 36));

        // test split with wrong span
        assert_eq!(span(24, 20).len(), 0);

        let splits6 = vec![
            Split::new(span(0, 16)),
            Split::new(span(16, 24)),
            Split::new(span(24, 16)),
            Split::new(span(24, 36)),
        ];

        let merged_splits6 = merge_splits(&splits6, max_tokens);
        assert_eq!(merged_splits6.len(), 2);
        assert_eq!(merged_splits6[0].tokens_span, span(0, 16));
        assert_eq!(merged_splits6[1].tokens_span, span(16, 36));
    }

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

    #[test]
    fn chunk_spans_test() {
        let single_span = vec![span(0, 10)];
        let chunked = chunk_spans(&single_span, 10);
        assert_eq!(chunked.len(), 1);
        assert_eq!(chunked[0].len(), 10);

        let even_spans = vec![span(0, 10), span(10, 16), span(16, 36), span(36, 45)];
        let chunked = chunk_spans(&even_spans, 20);
        assert_eq!(chunked.len(), 3);
        assert_eq!(chunked[0], span(0, 16));
        assert_eq!(chunked[0].len(), 16);
        assert_eq!(chunked[1], span(16, 36));
        assert_eq!(chunked[1].len(), 20);
        assert_eq!(chunked[2], span(36, 45));
        assert_eq!(chunked[2].len(), 9);

        let odd_spans = vec![
            span(0, 10),
            span(10, 16),
            span(16, 30),
            span(30, 35),
            span(35, 40),
        ];
        let chunked = chunk_spans(&odd_spans, 20);
        assert_eq!(chunked.len(), 3);
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
