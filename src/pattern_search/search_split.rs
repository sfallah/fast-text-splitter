use aho_corasick::Span;
use std::str::from_utf8;

#[derive(Clone, PartialEq, Eq)]
pub struct SearchSplit<'a> {
    pub data: &'a [u8],
    pub pattern: &'a str,
    pub stride: usize,
    pub span: Span,
}

impl std::fmt::Debug for SearchSplit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Split")
            .field("pattern", &self.pattern)
            .field("data", &self.data())
            .field("stride", &self.stride)
            .field("span", &self.span)
            .finish()
    }
}

impl<'a> SearchSplit<'a> {
    pub fn new(span: Span, data: &'a [u8], pattern: &'a str, stride: usize) -> Self {
        Self {
            data,
            pattern,
            stride,
            span,
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

    pub fn full_span(&self) -> Span {
        if self.stride == 1 {
            Span {
                start: self.span.start,
                end: self.span.end + self.pattern.len(),
            }
        } else {
            self.span
        }
    }
}
