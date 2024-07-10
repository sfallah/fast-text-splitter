use crate::common::tokens_result_lite::TokensResultLite;
use aho_corasick::Span;
use std::fmt;

#[derive(Clone)]
pub struct Split {
    pub pattern_id: usize,
    pub tokens_no: Option<usize>,
    pub data_span: Option<Span>,
    pub pattern_node: bool,
    pub tokens_results: Option<Vec<TokensResultLite>>,
}

impl fmt::Debug for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Split")
            .field("no_tokens", &format!("{:?}", &self.no_tokens()))
            .field("data_span", &format!("{:?}", &self.data_span))
            .field(
                "no_chars",
                &format!("{:?}", &self.data_span.map_or(0, |span| span.len())),
            )
            .finish()
    }
}

impl Split {
    pub fn no_tokens(&self) -> usize {
        if let Some(tokens_no) = self.tokens_no {
            tokens_no
        } else {
            // fall back to data span
            // this should only be the case for none-tokenizer (data len split)
            if let Some(data_span) = self.data_span {
                data_span.len()
            } else {
                0
            }
        }
    }
}
