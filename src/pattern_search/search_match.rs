use aho_corasick::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch<'a> {
    pub pattern: &'a str,
    pub span: Span,
}

impl SearchMatch<'_> {
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
