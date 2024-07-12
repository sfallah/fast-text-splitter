use aho_corasick::{AhoCorasick, Span};
use memchr::memmem::{find_iter, Finder};
use memchr::{memchr2_iter, memchr3_iter};

use crate::common::span;
use crate::pattern_search::get_aho_corasick;
use crate::pattern_search::search_match::SearchMatch;
use crate::pattern_search::search_pattern::SearchPattern;
use crate::pattern_search::search_result::SearchResult;
use crate::pattern_search::search_split::SearchSplit;


pub struct PatternSearcher {
    pub patterns: Vec<SearchPattern>,
    pub aho_corasick: Option<AhoCorasick>,
    pub memchr_finder: Option<Finder<'static>>,
}

impl PatternSearcher {
    pub fn new(str_patterns: Vec<String>) -> Self {
        let patterns: Vec<_> = str_patterns.clone().into_iter().map(SearchPattern::new).collect();
        if patterns.len() > 1 {
            if patterns.len() > 3 {
                let aho_corasick = Some(get_aho_corasick(str_patterns.clone()));
                Self {
                    patterns,
                    aho_corasick,
                    memchr_finder: None,
                }
            } else {
                // patterns.len() <= 3
                let all_single_byte = patterns.iter().all(|p| p.is_single_byte());
                if all_single_byte {
                    Self {
                        patterns,
                        aho_corasick: None,
                        memchr_finder: None,
                    }
                } else {
                    let aho_corasick = Some(get_aho_corasick(str_patterns.clone()));
                    Self {
                        patterns,
                        aho_corasick,
                        memchr_finder: None,
                    }
                }
            }
        } else {
            if patterns[0].len() > 1 {
                let memchr_finder = Some(Finder::new(patterns[0].as_bytes()).into_owned());
                Self {
                    patterns,
                    aho_corasick: None,
                    memchr_finder,
                }
            } else {
                Self {
                    patterns,
                    aho_corasick: None,
                    memchr_finder: None,
                }
            }
        }
    }
    pub fn search_patterns(&self, data: &[u8], data_span: Span) -> Vec<SearchMatch> {
        if self.patterns.is_empty() || data_span.is_empty() || data.is_empty() {
            return Vec::new();
        }
        if self.patterns.len() == 1 {
            let pattern = self.patterns.first().unwrap();
            if let Some(finder) = self.memchr_finder.as_ref() {
                finder
                    .find_iter(&data[data_span.range()])
                    .map(|start| SearchMatch {
                        pattern_len: pattern.len(),
                        is_pattern_whitespace: pattern.is_whitespace,
                        span: span(start, start + pattern.len()),
                    })
                    .collect()
            } else {
                find_iter(&data[data_span.range()], pattern.as_bytes())
                    .map(|start| SearchMatch {
                        pattern_len: pattern.len(),
                        is_pattern_whitespace: pattern.is_whitespace,
                        span: span(start, start + pattern.len()),
                    })
                    .collect()
            }
        } else {
            if let Some(ac) = self.aho_corasick.as_ref() {
                ac.find_iter(&data[data_span.range()])
                    .map(|mt| {
                        let pattern = self.patterns.get(mt.pattern().as_usize()).unwrap();
                        SearchMatch {
                            pattern_len: pattern.len(),
                            is_pattern_whitespace: pattern.is_whitespace,
                            span: mt.span().into(),
                        }
                    })
                    .collect()
            } else {
                if self.patterns.len() == 2 {
                    memchr2_iter(
                        self.patterns[0].as_bytes()[0],
                        self.patterns[1].as_bytes()[0],
                        &data[data_span.range()],
                    )
                        .map(|start| {
                            let is_pattern_whitespace = (data[start] as char).is_whitespace();
                            SearchMatch {
                                pattern_len: 1,
                                is_pattern_whitespace,
                                span: span(start, start + 1),
                            }
                        })
                        .collect()
                } else {
                    memchr3_iter(
                        self.patterns[0].as_bytes()[0],
                        self.patterns[1].as_bytes()[0],
                        self.patterns[2].as_bytes()[0],
                        &data[data_span.range()],
                    )
                        .map(|start| {
                            let is_pattern_whitespace = (data[start] as char).is_whitespace();
                            SearchMatch {
                                pattern_len: 1,
                                is_pattern_whitespace,
                                span: span(start, start + 1),
                            }
                        })
                        .collect()
                }
            }
        }
    }

    pub fn find_pattern<'a>(&'a self, data: &'a [u8], data_span: Span) -> SearchResult {
        let matches: Vec<_> = self.search_patterns(data, data_span);

        if matches.is_empty() {
            return SearchResult {
                data,
                offset: data_span.start,
                splits: vec![SearchSplit::new(Span::from(data_span), data, 0, false, 0)],
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
                cur_match.pattern_len,
                cur_match.is_pattern_whitespace,
                1,
            );
            pattern_splits.push(split);
        } else {
            // If the first match is at the beginning of the data
            let split = SearchSplit::new(
                span(data_span.start, data_span.start),
                data,
                cur_match.pattern_len,
                cur_match.is_pattern_whitespace,
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
                    cur_match.pattern_len,
                    cur_match.is_pattern_whitespace,
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
                cur_match.pattern_len,
                cur_match.is_pattern_whitespace,
                1,
            );
            pattern_splits.push(split);
            cur_match = end;
        });

        if cur_match.end() < data_span.len() {
            let split = SearchSplit::new(
                span(data_span.start + cur_match.end(), data_span.end),
                data,
                cur_match.pattern_len,
                cur_match.is_pattern_whitespace,
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
