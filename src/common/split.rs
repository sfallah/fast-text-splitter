use aho_corasick::Span;
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Split {
    pub tokens_span: Option<Span>,
    pub data_span: Option<Span>,
}

impl Split {
    pub fn new(tokens_span: Span) -> Self {
        Self {
            tokens_span: Some(tokens_span),
            data_span: None,
        }
    }
}

impl fmt::Debug for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Split")
            .field("tokens_span", &format!("{:?}", &self.tokens_span))
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
        if let Some(tokens_span) = self.tokens_span {
            tokens_span.len()
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
