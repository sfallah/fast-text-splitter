use aho_corasick::Span;
use std::fmt;

pub struct TokensResultLite {
    pub data_span: Span,
    pub ids: Option<Vec<u32>>,
    pub offsets: Option<Vec<(usize, usize)>>,
    pub pattern_id: usize,
}

impl Clone for TokensResultLite {
    fn clone(&self) -> Self {
        TokensResultLite {
            data_span: self.data_span.clone(),
            ids: self.ids.clone(),
            offsets: self.offsets.clone(),
            pattern_id: self.pattern_id,
        }
    }

}

impl TokensResultLite {
    pub fn no_tokens(&self) -> usize {
        if let Some(ids) = self.ids.as_ref() {
            ids.len()
        } else {
            if let Some(offsets) = self.offsets.as_ref() {
                offsets.len()
            } else {
                self.data_span.len()
            }
        }
    }
}

impl fmt::Debug for TokensResultLite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TokensResultLite {{ data_span: {:?}, ids: {:?}, offsets: {:?} }}",
            self.data_span,
            self.no_tokens(),
            self.offsets.as_ref().map_or_else(|| 0, |offsets| offsets.len())
        )
    }
}
