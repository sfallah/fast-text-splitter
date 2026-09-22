use crate::common::span;
use crate::pattern_search::get_aho_corasick;
use crate::pattern_search::search_match::SearchMatch;
use crate::pattern_search::search_pattern::SearchPattern;
use crate::pattern_search::search_result::SearchResult;
use crate::pattern_search::search_split::SearchSplit;
use aho_corasick::{AhoCorasick, Span};
use memchr::memmem::{find_iter, Finder};
use memchr::{memchr2_iter, memchr3_iter};
use sentence_splitter::{Language, Segmenter};

pub struct PatternSearcher {
    pub patterns: Vec<SearchPattern>,
    pub aho_corasick: Option<AhoCorasick>,
    pub memchr_finder: Option<Finder<'static>>,
    /// Set when the pattern is a sentence marker; it decides the boundaries itself.
    pub segmenter: Option<Segmenter>,
}

impl PatternSearcher {
    pub fn new(str_patterns: Vec<String>) -> Self {
        let patterns: Vec<_> = str_patterns
            .clone()
            .iter()
            .map(|pt| SearchPattern::new(pt.clone()))
            .collect();
        if patterns.len() > 1 {
            if patterns.len() > 3 {
                let aho_corasick = Some(get_aho_corasick(str_patterns.clone()));
                Self {
                    patterns,
                    aho_corasick,
                    memchr_finder: None,
                    segmenter: None,
                }
            } else {
                // patterns.len() <= 3
                let all_single_byte = patterns.iter().all(|p| p.is_single_byte());
                if all_single_byte {
                    Self {
                        patterns,
                        aho_corasick: None,
                        memchr_finder: None,
                        segmenter: None,
                    }
                } else {
                    let aho_corasick = Some(get_aho_corasick(str_patterns.clone()));
                    Self {
                        patterns,
                        aho_corasick,
                        memchr_finder: None,
                        segmenter: None,
                    }
                }
            }
        } else {
            // The sentence markers are literal strings longer than one byte, so they have
            // to be recognised before the length-based choice of search strategy.
            if patterns[0].is_icu_sentence {
                Self {
                    patterns,
                    aho_corasick: None,
                    memchr_finder: None,
                    segmenter: Some(Segmenter::icu()),
                }
            } else if patterns[0].is_sentence {
                Self {
                    patterns,
                    aho_corasick: None,
                    memchr_finder: None,
                    segmenter: Some(
                        Segmenter::punkt(Language::English)
                            .expect("the lang-english feature is enabled"),
                    ),
                }
            } else if patterns[0].len() > 1 {
                let memchr_finder = Some(Finder::new(patterns[0].as_bytes()).into_owned());
                Self {
                    patterns,
                    aho_corasick: None,
                    memchr_finder,
                    segmenter: None,
                }
            } else {
                Self {
                    patterns,
                    aho_corasick: None,
                    memchr_finder: None,
                    segmenter: None,
                }
            }
        }
    }
    pub fn search_patterns<'a>(
        &'a self,
        data: &'a [u8],
        data_span: Span,
    ) -> Box<dyn Iterator<Item = SearchMatch> + 'a> {
        if self.patterns.is_empty() || data_span.is_empty() || data.is_empty() {
            return Box::new(std::iter::empty());
        }
        if self.patterns.len() == 1 {
            let pattern = self.patterns.first().unwrap();
            if let Some(segmenter) = self.segmenter.as_ref() {
                // A sentence marker: the segmenter decides where sentences end, and each
                // end becomes a zero-width match.
                let data_slice = std::str::from_utf8(&data[data_span.range()]).unwrap();
                let ends = segmenter
                    .boundaries(data_slice)
                    .expect("segmenter was constructed with an available backend");
                Box::new(ends.into_iter().map(|end| SearchMatch {
                    pattern_len: 0,
                    is_pattern_whitespace: true,
                    span: span(end, end),
                }))
            } else {
                if let Some(finder) = self.memchr_finder.as_ref() {
                    Box::new(
                        finder
                            .find_iter(&data[data_span.range()])
                            .map(move |start| SearchMatch {
                                pattern_len: pattern.len(),
                                is_pattern_whitespace: pattern.is_whitespace,
                                span: span(start, start + pattern.len()),
                            }),
                    )
                } else {
                    Box::new(find_iter(&data[data_span.range()], pattern.as_bytes()).map(
                        move |start| SearchMatch {
                            pattern_len: pattern.len(),
                            is_pattern_whitespace: pattern.is_whitespace,
                            span: span(start, start + pattern.len()),
                        },
                    ))
                }
            }
        } else {
            if let Some(ac) = self.aho_corasick.as_ref() {
                Box::new(ac.find_iter(&data[data_span.range()]).map(move |mt| {
                    let pattern = self.patterns.get(mt.pattern().as_usize()).unwrap();
                    SearchMatch {
                        pattern_len: pattern.len(),
                        is_pattern_whitespace: pattern.is_whitespace,
                        span: mt.span().into(),
                    }
                }))
            } else {
                if self.patterns.len() == 2 {
                    Box::new(
                        memchr2_iter(
                            self.patterns[0].as_bytes()[0],
                            self.patterns[1].as_bytes()[0],
                            &data[data_span.range()],
                        )
                        .map(move |start| {
                            let is_pattern_whitespace =
                                (data[data_span.start + start] as char).is_whitespace();
                            SearchMatch {
                                pattern_len: 1,
                                is_pattern_whitespace,
                                span: span(start, start + 1),
                            }
                        }),
                    )
                } else {
                    Box::new(
                        memchr3_iter(
                            self.patterns[0].as_bytes()[0],
                            self.patterns[1].as_bytes()[0],
                            self.patterns[2].as_bytes()[0],
                            &data[data_span.range()],
                        )
                        .map(move |start| {
                            let is_pattern_whitespace =
                                (data[data_span.start + start] as char).is_whitespace();
                            SearchMatch {
                                pattern_len: 1,
                                is_pattern_whitespace,
                                span: span(start, start + 1),
                            }
                        }),
                    )
                }
            }
        }
    }

    pub fn find_pattern<'a>(&'a self, data: &'a [u8], data_span: Span) -> SearchResult<'a> {
        let mut matches = self.search_patterns(data, data_span);

        let first_match_opt = matches.next();

        if first_match_opt.is_none() {
            return SearchResult {
                data,
                offset: data_span.start,
                splits: vec![SearchSplit::new(Span::from(data_span), data, 0, false, 0)],
                matched: false,
            };
        }

        let mut pattern_splits: Vec<SearchSplit> = Vec::new();

        let mut cur_match = first_match_opt.unwrap();

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

        matches.for_each(|next_match| {
            let search_split = SearchSplit::new(
                span(
                    data_span.start + cur_match.end(),
                    data_span.start + next_match.start(),
                ),
                data,
                next_match.pattern_len,
                next_match.is_pattern_whitespace,
                1,
            );
            pattern_splits.push(search_split);
            cur_match = next_match;
        });

        // If the pattern is not at the end of the data
        if cur_match.end() < data_span.len() {
            let search_split = SearchSplit::new(
                span(data_span.start + cur_match.end(), data_span.end),
                data,
                cur_match.pattern_len,
                false,
                0,
            );
            pattern_splits.push(search_split);
        }

        SearchResult {
            data,
            offset: data_span.start,
            splits: pattern_splits,
            matched: true,
        }
    }
}
