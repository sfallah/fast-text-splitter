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
    pub splits: Split,
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
