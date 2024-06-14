use aho_corasick::{AhoCorasick, Span};
use memchr::memmem::find_iter;

use crate::common::span;
use crate::pattern_search::get_aho_corasick;
use crate::pattern_search::search_match::SearchMatch;
use crate::pattern_search::search_result::SearchResult;
use crate::pattern_search::search_split::SearchSplit;

pub struct PatternSearcher<'a> {
    pub patterns: &'a Vec<&'a str>,
    pub engine: Option<AhoCorasick>,
}

impl<'a> PatternSearcher<'a> {
    pub fn new(patterns: &'a Vec<&str>) -> Self {
        let engine = if patterns.len() > 1 {
            Some(get_aho_corasick(patterns))
        } else {
            None
        };
        Self { patterns, engine }
    }

    pub fn search_patterns(&self, data: &'a [u8], data_span: Span) -> Vec<SearchMatch<'a>> {
        if self.patterns.is_empty() || data_span.is_empty() || data.is_empty() {
            return Vec::new();
        }

        if self.patterns.len() == 1 {
            let pattern = self.patterns.first().unwrap();
            let pattern_len = pattern.len();
            find_iter(&data[data_span.start..data_span.end], pattern.as_bytes())
                .map(|start| SearchMatch {
                    pattern,
                    span: span(start, start + pattern_len),
                })
                .collect()
        } else {
            let ac = self.engine.as_ref().unwrap();
            ac.find_iter(&data[data_span.start..data_span.end])
                .map(|mt| SearchMatch {
                    pattern: &self.patterns[mt.pattern().as_usize()],
                    span: mt.span().into(),
                })
                .collect()
        }
    }

    pub fn find_pattern(&self, data: &'a [u8], data_span: Span) -> SearchResult<'a> {
        let matches: Vec<_> = self.search_patterns(data, data_span);

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
            if cur_match.end() < data_span.len() {
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

        if cur_match.end() < data_span.len() {
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
}
