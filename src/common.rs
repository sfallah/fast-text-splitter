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

#[derive(PartialEq, Debug, Clone)]
pub struct SplitResults {
    pub split: Split,
    #[cfg(feature = "tokenizers")]
    pub results: Option<TokensResults>,
    pub split_strings: String,
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct TokensResults {
    pub ids: Vec<u32>,
    pub type_ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub offsets: Vec<(usize, usize)>,
}

impl TokensResults {
    pub fn extend(&mut self, other: TokensResults) {
        self.ids.extend(other.ids);
        self.type_ids.extend(other.type_ids);
        self.attention_mask.extend(other.attention_mask);
        self.offsets.extend(other.offsets);
    }
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
}
