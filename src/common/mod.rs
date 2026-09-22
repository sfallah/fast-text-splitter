use aho_corasick::Span;

pub mod split;
pub mod tokens_result_lite;

#[inline]
pub fn span(start: usize, end: usize) -> Span {
    assert!(
        start <= end,
        "Start: {} must be less than or equal to End: {}",
        start,
        end
    );
    Span { start, end }
}

// negative offset the span
pub fn span_ne_offset(span: &Span, offset: usize) -> Span {
    let start = span.start - offset;
    let end = span.end - offset;
    Span { start, end }
}
