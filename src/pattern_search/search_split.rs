use aho_corasick::Span;
use std::str::from_utf8;

#[derive(Clone, PartialEq, Eq)]
pub struct SearchSplit<'a> {
    pub data: &'a [u8],
    pub stride: usize,
    pub span: Span,
    pub pattern_len: usize,
    pub is_pattern_whitespace: bool,
}

impl std::fmt::Debug for SearchSplit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Split")
            .field("data", &self.data())
            .field("stride", &self.stride)
            .field("span", &self.span)
            .finish()
    }
}

impl<'a> SearchSplit<'a> {
    pub fn new(
        span: Span,
        data: &'a [u8],
        pattern_len: usize,
        is_pattern_whitespace: bool,
        stride: usize,
    ) -> Self {
        Self {
            data,
            stride,
            span,
            pattern_len,
            is_pattern_whitespace,
        }
    }
    pub fn data(&self) -> String {
        from_utf8(&self.data[self.span.start..self.span.end])
            .unwrap()
            .to_string()
    }

    pub fn reconstruct(&self) -> String {
        let full_span = self.full_span();
        from_utf8(&self.data[full_span.start..full_span.end])
            .unwrap()
            .to_string()
    }

    pub fn pattern_len(&self) -> usize {
        self.pattern_len
    }

    pub fn full_span(&self) -> Span {
        if self.stride == 1 {
            Span {
                start: self.span.start,
                end: self.span.end + self.pattern_len(),
            }
        } else {
            self.span
        }
    }
}
