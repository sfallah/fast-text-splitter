use aho_corasick::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub pattern_len: usize,
    pub span: Span,
}

impl SearchMatch {
    pub fn len(&self) -> usize {
        self.span.len()
    }
    pub fn start(&self) -> usize {
        self.span.start
    }
    pub fn end(&self) -> usize {
        self.span.end
    }
}
