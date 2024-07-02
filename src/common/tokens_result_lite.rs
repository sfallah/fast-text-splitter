use std::fmt;

#[derive(Clone)]
pub struct TokensResultLite {
    pub ids: Option<Vec<u32>>,
    pub offsets: Option<Vec<(usize, usize)>>,
}

impl Default for TokensResultLite {
    fn default() -> Self {
        TokensResultLite {
            ids: None,
            offsets: None,
        }
    }
}

impl TokensResultLite {
    pub fn no_tokens(&self) -> usize {
        if let Some(ids) = self.ids.as_ref() {
            ids.len()
        } else {
            0
        }
    }
}

impl fmt::Debug for TokensResultLite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TokensResultLite {{ ids: {:?}, offsets: {:?} }}",
            self.no_tokens(),
            self.offsets.as_ref().map_or_else(|| 0, |offsets| offsets.len())
        )
    }
}
