use crate::common::span;
use crate::pattern_search::get_aho_corasick;
use crate::pattern_search::search_match::SearchMatch;
use crate::pattern_search::search_pattern::SearchPattern;
use crate::pattern_search::search_result::SearchResult;
use crate::pattern_search::search_split::SearchSplit;
use aho_corasick::{AhoCorasick, Span};
use icu_segmenter::options::SentenceBreakInvariantOptions;
use icu_segmenter::{SentenceSegmenter, SentenceSegmenterBorrowed};
use itertools::Itertools;
use memchr::memmem::{find_iter, Finder};
use memchr::{memchr2_iter, memchr3_iter};
use once_cell::sync::Lazy;
use punkt::{sentence_tokenize, sentence_tokenize_lang, SentenceToken};

static ICU_SENTENCE_TOKENIZER: Lazy<SentenceSegmenterBorrowed> =
    Lazy::new(|| SentenceSegmenter::new(SentenceBreakInvariantOptions::default()));

pub struct PatternSearcher {
    pub patterns: Vec<SearchPattern>,
    pub aho_corasick: Option<AhoCorasick>,
    pub memchr_finder: Option<Finder<'static>>,
    pub sentence_tokenizer: Option<bool>,
    pub icu_tokenizer: Option<bool>,
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
                    sentence_tokenizer: None,
                    icu_tokenizer: None,
                }
            } else {
                // patterns.len() <= 3
                let all_single_byte = patterns.iter().all(|p| p.is_single_byte());
                if all_single_byte {
                    Self {
                        patterns,
                        aho_corasick: None,
                        memchr_finder: None,
                        sentence_tokenizer: None,
                        icu_tokenizer: None,
                    }
                } else {
                    let aho_corasick = Some(get_aho_corasick(str_patterns.clone()));
                    Self {
                        patterns,
                        aho_corasick,
                        memchr_finder: None,
                        sentence_tokenizer: None,
                        icu_tokenizer: None,
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
                    sentence_tokenizer: None,
                    icu_tokenizer: None,
                }
            } else {
                if patterns[0].is_icu_sentence {
                    // If the pattern is a whitespace, we can use a memchr finder
                    Self {
                        patterns,
                        aho_corasick: None,
                        memchr_finder: None,
                        sentence_tokenizer: None,
                        icu_tokenizer: Some(true),
                    }
                } else if patterns[0].is_sentence {
                    Self {
                        patterns,
                        aho_corasick: None,
                        memchr_finder: None,
                        sentence_tokenizer: Some(true),
                        icu_tokenizer: None,
                    }
                } else {
                    Self {
                        patterns,
                        aho_corasick: None,
                        memchr_finder: None,
                        sentence_tokenizer: None,
                        icu_tokenizer: None,
                    }
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
            if pattern.is_icu_sentence {
                let data_slice = std::str::from_utf8(&data[data_span.range()]).unwrap();
                let sentences =
                    icu_sentence_tokenize(data_slice)
                        .unwrap()
                        .into_iter()
                        .map(move |st| SearchMatch {
                            pattern_len: 0,
                            is_pattern_whitespace: true,
                            span: span(st.span.end, st.span.end),
                        });
                Box::new(sentences)
            } else if pattern.is_sentence {
                let data_slice = std::str::from_utf8(&data[data_span.range()]).unwrap();
                Box::new(
                    sentence_tokenize_lang(data_slice, Some("english"))
                        .unwrap()
                        .into_iter()
                        .map(move |st| SearchMatch {
                            pattern_len: 0,
                            is_pattern_whitespace: true,
                            span: span(st.span.end, st.span.end),
                        }),
                )
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

pub fn icu_sentence_tokenize(text: &str) -> anyhow::Result<Vec<SentenceToken>> {
    let mut cleaned_text = Vec::new();
    replace_newlines_with_space(text, &mut cleaned_text)
        .map_err(|e| anyhow::anyhow!("Failed to replace newlines: {}", e))?;
    // Collect byte offsets produced by ICU.
    let offsets: Vec<_> = ICU_SENTENCE_TOKENIZER
        .segment_utf8(&cleaned_text)
        .tuple_windows()
        .map(|(start, end)| {
            // SAFETY: offsets are byte indices from ICU; this must be valid UTF-8 boundaries.
            (start, end)
        })
        .collect();

    // Ensure we have a leading 0 and trailing text.len() to form closed intervals.

    // Slice directly from offset pairs.
    let mut icu_sentences = Vec::new();
    for (start, end) in offsets {
        // SAFETY: offsets are byte indices from ICU; this must be valid UTF-8 boundaries.
        let sentence_str = &text[start..end];
        icu_sentences.push(SentenceToken {
            span: punkt::Span { start, end },
            text: sentence_str.to_string(),
        });
    }

    Ok(icu_sentences)
}

static AHO_CORASICK: Lazy<AhoCorasick> = Lazy::new(|| AhoCorasick::new(&["\n"]).unwrap());

pub fn replace_newlines_with_space(rdr: &str, wtr: &mut Vec<u8>) -> anyhow::Result<()> {
    AHO_CORASICK
        .try_stream_replace_all(rdr.as_bytes(), wtr, &[" "])
        .map_err(|e| anyhow::anyhow!("Failed to replace newlines: {}", e))?;
    Ok(())
}
