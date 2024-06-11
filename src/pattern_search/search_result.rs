use crate::pattern_search::search_split::SearchSplit;
use aho_corasick::Span;

#[derive(Clone, PartialEq, Eq)]
pub struct SearchResult<'a> {
    pub data: &'a [u8],
    pub offset: usize,
    pub splits: Vec<SearchSplit<'a>>,
    pub matched: bool,
}

// impl From for SearchResult

impl<'a> From<SearchSplit<'a>> for SearchResult<'a> {
    fn from(split: SearchSplit<'a>) -> Self {
        Self {
            data: split.data,
            offset: split.span.start,
            splits: vec![split],
            matched: false,
        }
    }
}

impl<'a> SearchResult<'a> {
    pub fn data(&self) -> Vec<String> {
        self.splits.iter().map(|split| split.data()).collect()
    }
    pub fn offsets(&self) -> Vec<Span> {
        self.splits.iter().map(|split| split.span).collect()
    }
}

impl std::fmt::Debug for SearchResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchResult")
            .field("offset", &self.offset)
            .field("splits", &self.splits)
            .field("matched", &self.matched)
            .finish()
    }
}
