use std::fmt;
use aho_corasick::Span;

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