use std::str::from_utf8;

use aho_corasick::{AhoCorasick, MatchKind, Span};

pub mod pattern_searcher;

pub mod search_match;
pub mod search_result;
pub mod search_split;
mod tests;

pub fn span_within_bounds(span: Span, data_len: usize) -> bool {
    // the span is valid
    span.start <= span.end &&
        // the span is within the bounds of the data
        span.start < data_len && span.end <= data_len
}

pub fn span_to_string(data: &str, span: Span) -> Option<String> {
    if span_within_bounds(span, data.len()) {
        if span.is_empty() {
            Some("".to_string())
        } else {
            Some(
                from_utf8(&data.as_bytes()[span.start..span.end])
                    .unwrap()
                    .to_string(),
            )
        }
    } else {
        None
    }
}

pub fn get_aho_corasick(patterns: &Vec<&str>) -> AhoCorasick {
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)
        .unwrap()
}
