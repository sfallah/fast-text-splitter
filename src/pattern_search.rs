use std::str::from_utf8;

use aho_corasick::{AhoCorasick, MatchKind, Span};
use memchr::memmem::find_iter;

use crate::common::span;

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

    pub fn reconstruct(&self) -> String {
        let full_span = self.full_span();
        from_utf8(&self.data[full_span.start..full_span.end])
            .unwrap()
            .to_string()
    }
    pub fn full_span(&self) -> Span {
        let start = self.offset;
        let end = self.splits[self.splits.len() - 1].full_span().end;
        Span { start, end }
    }

    pub fn len(&self) -> usize {
        self.splits.iter().map(|split| split.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.splits.iter().all(|split| split.is_empty())
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

#[derive(Clone, PartialEq, Eq)]
pub struct SearchSplit<'a> {
    pub data: &'a [u8],
    pub pattern: &'a str,
    pub stride: usize,
    pub span: Span,
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
    pub fn len(&self) -> usize {
        self.span.len()
    }
    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
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

pub fn get_aho_corasick(patterns: &Vec<&str>) -> AhoCorasick {
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)
        .unwrap()
}

pub fn search_patterns<'a>(
    patterns: &Vec<&'a str>,
    data: &'a [u8],
    data_span: Span,
) -> Vec<SearchMatch<'a>> {
    if patterns.is_empty() || data_span.is_empty() || data.is_empty() {
        return Vec::new();
    }

    if patterns.len() == 1 {
        let pattern = patterns.first().unwrap();
        let pattern_len = pattern.len();
        find_iter(&data[data_span.start..data_span.end], pattern.as_bytes())
            .map(|start| SearchMatch {
                pattern,
                span: span(start, start + pattern_len),
            })
            .collect()
    } else {
        let ac = get_aho_corasick(patterns);
        ac.find_iter(&data[data_span.start..data_span.end])
            .map(|mt| SearchMatch {
                pattern: &patterns[mt.pattern().as_usize()],
                span: mt.span().into(),
            })
            .collect()
    }
}

pub fn find_pattern<'a>(
    patterns: &Vec<&'a str>,
    data: &'a [u8],
    data_span: Span,
) -> SearchResult<'a> {
    let matches: Vec<_> = search_patterns(patterns, data, data_span);

    if matches.is_empty() {
        return SearchResult {
            data,
            offset: data_span.start,
            splits: vec![SearchSplit::new(Span::from(data_span), data, "", 0)],
            matched: false,
        };
    }

    let mut pattern_splits: Vec<SearchSplit> = Vec::new();

    let mut cur_match = matches.first().unwrap();

    // If the first match is not at the beginning of the data
    if cur_match.start() > 0 {
        let split = SearchSplit::new(
            span(data_span.start, data_span.start + cur_match.start()),
            data,
            cur_match.pattern,
            1,
        );
        pattern_splits.push(split);
    } else {
        // If the first match is at the beginning of the data
        let split = SearchSplit::new(
            span(data_span.start, data_span.start),
            data,
            cur_match.pattern,
            1,
        );

        pattern_splits.push(split);
    }
    //cur_match += pattern_len;

    if matches.len() == 1 {
        // If the pattern is not at the end of the data
        if cur_match.end() < data_span.len() - 1 {
            let split = SearchSplit::new(
                span(data_span.start + cur_match.end(), data_span.end),
                data,
                cur_match.pattern,
                0,
            );

            pattern_splits.push(split);
        }
        return SearchResult {
            data,
            offset: data_span.start,
            splits: pattern_splits,
            matched: true,
        };
    }

    matches.iter().skip(1).for_each(|end| {
        let split = SearchSplit::new(
            span(
                data_span.start + cur_match.end(),
                data_span.start + end.start(),
            ),
            data,
            cur_match.pattern,
            1,
        );
        pattern_splits.push(split);
        cur_match = end;
    });

    if cur_match.end() < data_span.len() - 1 {
        let split = SearchSplit::new(
            span(data_span.start + cur_match.end(), data_span.end),
            data,
            cur_match.pattern,
            0,
        );
        pattern_splits.push(split);
    }

    SearchResult {
        data,
        offset: data_span.start,
        splits: pattern_splits,
        matched: true,
    }
}

