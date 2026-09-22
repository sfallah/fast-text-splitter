#[cfg(test)]
mod tests {
    use aho_corasick::Span;

    use crate::common::span;
    use crate::pattern_search::pattern_searcher::PatternSearcher;
    use crate::pattern_search::search_result::SearchResult;
    use crate::pattern_search::span_to_string;
    use crate::test_support::{read, LINES, SHORT};

    const SENTENCE_MARKERS: [&str; 2] = ["<SENT>", "<ICU_SENT>"];

    fn searcher(patterns: &[&str]) -> PatternSearcher {
        PatternSearcher::new(patterns.iter().map(|p| p.to_string()).collect())
    }

    fn find<'a>(searcher: &'a PatternSearcher, data: &'a [u8]) -> SearchResult<'a> {
        searcher.find_pattern(data, span(0, data.len()))
    }

    /// The text between separators and the stride of every split, in order.
    fn parts(result: &SearchResult) -> Vec<(String, usize)> {
        result.splits.iter().map(|s| (s.data(), s.stride)).collect()
    }

    fn match_spans(searcher: &PatternSearcher, data: &[u8]) -> Vec<Span> {
        searcher
            .search_patterns(data, span(0, data.len()))
            .map(|m| m.span)
            .collect()
    }

    /// The splits' `full_span`s must be contiguous and cover exactly the searched span.
    fn assert_tiles(result: &SearchResult, data_span: Span) {
        let mut cursor = data_span.start;
        for split in &result.splits {
            let full = split.full_span();
            assert_eq!(full.start, cursor, "gap or overlap before {split:?}");
            cursor = full.end;
        }
        assert_eq!(
            cursor, data_span.end,
            "splits stop short of the searched span"
        );
    }

    #[test]
    fn search_patterns_yields_leftmost_first_matches() {
        let greeting = "Hello, you all!\n\n How are you? \n\n".as_bytes();
        let cases: [(&[&str], &[u8], Vec<Span>); 4] = [
            (&["\n\n"], greeting, vec![span(15, 17), span(31, 33)]),
            // The first listed pattern wins where both could match.
            (&["\n\n", "\n"], greeting, vec![span(15, 17), span(31, 33)]),
            (
                &["\n", "\n\n"],
                greeting,
                vec![span(15, 16), span(16, 17), span(31, 32), span(32, 33)],
            ),
            (
                &[".", "!", "?"],
                b"Hello, you all! How are you? Nice to be here.",
                vec![span(14, 15), span(27, 28), span(44, 45)],
            ),
        ];
        for (patterns, data, expected) in cases {
            assert_eq!(
                match_spans(&searcher(patterns), data),
                expected,
                "{patterns:?}"
            );
        }

        let matches: Vec<_> = searcher(&["\n\n"])
            .search_patterns(greeting, span(0, greeting.len()))
            .collect();
        assert!(matches
            .iter()
            .all(|m| m.pattern_len == 2 && m.is_pattern_whitespace));
    }

    #[test]
    fn no_match_returns_the_whole_input_unmatched() {
        let searcher = searcher(&["\n\n"]);
        for data in ["Hello, you all! How are you?", ""] {
            let result = find(&searcher, data.as_bytes());
            assert!(!result.matched);
            assert_eq!(parts(&result), [(data.to_string(), 0)]);
        }
    }

    #[test]
    fn blank_line_separators_at_start_middle_and_end() {
        let cases: [(&str, Vec<(&str, usize)>); 9] = [
            ("Hello, you all!\n\nHow are you ?", vec![("Hello, you all!", 1), ("How are you ?", 0)]),
            (
                "Hello, you all!\n\n How are you? \n\n",
                vec![("Hello, you all!", 1), (" How are you? ", 1)],
            ),
            (
                "Hello, you all!\n\n How are you? \n\n Nice to meet you all!",
                vec![("Hello, you all!", 1), (" How are you? ", 1), (" Nice to meet you all!", 0)],
            ),
            (
                "\n\nHello, you all! How are you? \n\n",
                vec![("", 1), ("Hello, you all! How are you? ", 1)],
            ),
            (
                "\n\nHello, you all!\n\n How are you? \n\n Nice to meet you all!\n\n I hope you are all doing well!",
                vec![
                    ("", 1),
                    ("Hello, you all!", 1),
                    (" How are you? ", 1),
                    (" Nice to meet you all!", 1),
                    (" I hope you are all doing well!", 0),
                ],
            ),
            // Runs of separators produce empty splits between them.
            (
                "Hello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n",
                vec![("Hello, you all!", 1), (" How are you? ", 1), ("", 1), (" Nice to meet you all!", 1)],
            ),
            (
                "\n\n\n\nHello, you all! How are you? \n\n\n\n",
                vec![("", 1), ("", 1), ("Hello, you all! How are you? ", 1), ("", 1)],
            ),
            (
                "\n\n\n\nHello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n\n\n",
                vec![
                    ("", 1),
                    ("", 1),
                    ("Hello, you all!", 1),
                    (" How are you? ", 1),
                    ("", 1),
                    (" Nice to meet you all!", 1),
                    ("", 1),
                ],
            ),
            (
                SHORT,
                vec![
                    ("", 1),
                    ("Returns and inequality, is so strong.\nThat it yields.", 1),
                    ("Another heuristic for.", 1),
                    ("\nOutperform everyone else.", 1),
                ],
            ),
        ];
        let searcher = searcher(&["\n\n"]);
        for (data, expected) in cases {
            let result = find(&searcher, data.as_bytes());
            assert!(result.matched, "{data:?}");
            let expected: Vec<(String, usize)> = expected
                .into_iter()
                .map(|(s, n)| (s.to_string(), n))
                .collect();
            assert_eq!(parts(&result), expected, "{data:?}");
            assert_tiles(&result, span(0, data.len()));
        }
    }

    #[test]
    fn single_newline_separator() {
        let searcher = searcher(&["\n"]);
        let two_lines =
            "In fact, the correlation. Between superlinear.\nReturns and inequality is so strong that it yields.";
        let result = find(&searcher, two_lines.as_bytes());
        assert_eq!(
            parts(&result),
            [
                (
                    "In fact, the correlation. Between superlinear.".to_string(),
                    1
                ),
                (
                    "Returns and inequality is so strong that it yields.".to_string(),
                    0
                ),
            ]
        );

        let one_line = "Returns and inequality is so strong that it yields.";
        let result = find(&searcher, one_line.as_bytes());
        assert!(!result.matched);
        assert_eq!(result.splits.len(), 1);
    }

    #[test]
    fn separator_at_the_edges_of_the_searched_span() {
        let searcher = searcher(&["\n\n"]);
        let text = "Hello, you all! How are you?";
        // The same layout with and without bytes before the searched span.
        for prefix in ["", "<from before NOT IN SEARCH>\n\n"] {
            let offset = prefix.len();

            let leading = format!("{prefix}\n\n{text}");
            let result = searcher.find_pattern(leading.as_bytes(), span(offset, leading.len()));
            assert_eq!(result.splits.len(), 2, "{leading:?}");
            let first = &result.splits[0];
            assert_eq!(first.span, span(offset, offset));
            assert_eq!(first.stride, 1);
            assert_eq!(first.data(), "");
            assert_eq!(first.reconstruct(), "\n\n");
            assert!(first.is_pattern_whitespace);
            let last = &result.splits[1];
            assert_eq!(last.span.len(), text.len());
            assert_eq!(last.stride, 0);
            assert_eq!(last.data(), text);
            assert_eq!(last.reconstruct(), text);
            assert!(!last.is_pattern_whitespace);
            assert_tiles(&result, span(offset, leading.len()));

            let trailing = format!("{prefix}{text}\n\n");
            let result = searcher.find_pattern(trailing.as_bytes(), span(offset, trailing.len()));
            assert_eq!(result.splits.len(), 1, "{trailing:?}");
            let only = &result.splits[0];
            assert_eq!(only.span.len(), text.len());
            assert_eq!(only.stride, 1);
            assert_eq!(only.data(), text);
            assert_eq!(only.reconstruct(), format!("{text}\n\n"));
            assert_tiles(&result, span(offset, trailing.len()));
        }
    }

    #[test]
    fn punctuation_separators_are_not_whitespace() {
        let searcher = searcher(&[".", "!", "?"]);

        let result = find(&searcher, b"Hello, you all! How are you");
        assert_eq!(result.splits.len(), 2);
        assert_eq!(result.splits[0].span, span(0, 14));
        assert_eq!(result.splits[0].stride, 1);
        assert_eq!(result.splits[0].data(), "Hello, you all");
        assert_eq!(result.splits[0].reconstruct(), "Hello, you all!");
        assert_eq!(result.splits[1].span, span(15, 27));
        assert_eq!(result.splits[1].stride, 0);
        assert_eq!(result.splits[1].data(), " How are you");
        assert!(result.splits.iter().all(|s| !s.is_pattern_whitespace));

        let result = find(&searcher, b"Hello, you all");
        assert!(!result.matched);
        assert_eq!(result.splits[0].span, span(0, 14));
        assert_eq!(result.splits[0].stride, 0);

        let bang = self::searcher(&["!"]);
        let result = find(&bang, b"Hello, you all!0");
        assert_eq!(
            parts(&result),
            [("Hello, you all".to_string(), 1), ("0".to_string(), 0)]
        );
    }

    #[test]
    fn splits_tile_real_files() {
        let cases: [(&[&str], &str, Option<usize>); 3] = [
            (
                &["\n\n"],
                "tests/splitter_test_data/data_nlnl_01.txt",
                Some(8),
            ),
            (&["\n"], "tests/error_data/nbsp_lines.txt", None),
            (
                &[".", "!", "?"],
                "tests/error_data/wiki_us_snippet_error.txt",
                None,
            ),
        ];
        for (patterns, path, expected_splits) in cases {
            let data = read(path);
            let searcher = self::searcher(patterns);
            let result = find(&searcher, &data);
            assert_tiles(&result, span(0, data.len()));
            if let Some(expected) = expected_splits {
                assert_eq!(result.splits.len(), expected, "{path}");
            }
        }
    }

    #[test]
    fn span_to_string_rejects_out_of_bounds_spans() {
        let data = "Hello, you all! How are you?";
        assert_eq!(span_to_string(data, span(5, 5)), Some("".to_string()));
        assert_eq!(span_to_string(data, span(5, 10)), Some(", you".to_string()));
        assert_eq!(
            span_to_string(data, span(0, data.len())),
            Some(data.to_string())
        );
        assert_eq!(span_to_string(data, Span { start: 5, end: 2 }), None);
        assert_eq!(
            span_to_string(data, span(data.len() - 3, data.len() + 1)),
            None
        );
    }

    // Both sentence segmenters must agree on these inputs.

    #[test]
    fn sentence_markers_are_recognized() {
        let searcher = searcher(&["<SENT>"]);
        assert!(searcher.patterns[0].is_sentence && !searcher.patterns[0].is_icu_sentence);
        let searcher = self::searcher(&["<ICU_SENT>"]);
        assert!(searcher.patterns[0].is_icu_sentence && !searcher.patterns[0].is_sentence);

        for marker in SENTENCE_MARKERS {
            let data = b"Hello, you all! How are you?";
            let searcher = self::searcher(&[marker]);
            let result = find(&searcher, data);
            assert_eq!(result.splits.len(), 2, "{marker}");
            assert_tiles(&result, span(0, data.len()));
        }
    }

    #[test]
    fn sentence_markers_split_line_and_paragraph_text() {
        let across_blank_lines = "\n\
            In fact, the correlation. Between superlinear.\n\
            Returns and inequality is \n\n so strong that it yields.\n\
            Outperform everyone else\n\n\
            just like that\n";
        for marker in SENTENCE_MARKERS {
            let searcher = searcher(&[marker]);
            for (data, expected) in [(LINES, 8), (across_blank_lines, 4)] {
                let result = find(&searcher, data.as_bytes());
                assert_eq!(result.splits.len(), expected, "{marker} on {data:?}");
                assert_tiles(&result, span(0, data.len()));
            }
        }
    }

    #[test]
    fn sentence_markers_on_empty_and_whitespace_input() {
        for marker in SENTENCE_MARKERS {
            let searcher = searcher(&[marker]);

            let result = find(&searcher, b"");
            assert_eq!(result.splits.len(), 1, "{marker}");
            assert!(result.splits[0].span.is_empty());

            let result = find(&searcher, b"\n\n");
            assert_eq!(result.splits.len(), 1, "{marker}");
            assert_eq!(result.splits[0].data(), "\n\n");
        }
    }

    #[test]
    fn punkt_keeps_unpunctuated_pdf_extract_as_one_sentence() {
        let data = read("tests/error_data/pdf_extract_error.txt");
        let searcher = self::searcher(&["<SENT>"]);
        let result = find(&searcher, &data);
        assert_eq!(result.splits.len(), 1);
    }
}
