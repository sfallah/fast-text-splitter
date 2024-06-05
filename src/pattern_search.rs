use std::str::from_utf8;

use aho_corasick::Span;
use memchr::memmem::find_iter;
use crate::span;

#[derive(Clone, PartialEq, Eq)]
pub struct SearchResult<'a> {
    pub data: &'a [u8],
    pub offset: usize,
    pub pattern: &'a str,
    pub splits: Vec<SearchSplit<'a>>,
    pub matched: bool,
}

// impl From for SearchResult

impl<'a> From<SearchSplit<'a>> for SearchResult<'a> {
    fn from(split: SearchSplit<'a>) -> Self {
        Self {
            data: split.data,
            offset: split.span.start,
            pattern: split.pattern,
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
        let start = self.splits[0].full_span().start;
        let end = self.splits[self.splits.len() - 1].full_span().end;
        Span { start, end }
    }
}

impl std::fmt::Debug for SearchResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchResult")
            .field("offset", &self.offset)
            .field("pattern", &self.pattern)
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

fn span_within_bounds(span: Span, data_len: usize) -> bool {
    // the span is valid
    span.start <= span.end &&
        // the span is within the bounds of the data
        span.start < data_len && span.end <= data_len
}

fn span_to_string(data: &str, span: Span) -> Option<String> {
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

pub fn find_pattern<'a>(pattern: &'a str, data: &'a [u8], data_span: Span) -> SearchResult<'a> {
    let pattern_len = pattern.len();
    let matches: Vec<_> =
        find_iter(&data[data_span.start..data_span.end], pattern.as_bytes()).collect();

    if matches.is_empty() {
        return SearchResult {
            data,
            offset: data_span.start,
            pattern,
            splits: vec![SearchSplit::new(
                span(data_span.start,
                data_span.end),
                data,
                pattern,
                0,
            )],
            matched: false,
        };
    }

    let mut pattern_splits: Vec<SearchSplit> = Vec::new();

    let mut start = matches[0];

    // If the first match is not at the beginning of the data
    if start > 0 {
        let split = SearchSplit::new(span(data_span.start, data_span.start + start), data, pattern, 1);
        pattern_splits.push(split);
    } else {
        // If the first match is at the beginning of the data
        let split = SearchSplit::new(span(data_span.start, data_span.start), data, pattern, 1);

        pattern_splits.push(split);
    }
    start += pattern_len;

    if matches.len() == 1 {
        // If the pattern is not at the end of the data
        if start < data.len() - 1 {
            let split = SearchSplit::new(span(data_span.start + start, data_span.end), data, pattern, 0);

            pattern_splits.push(split);
        }
        return SearchResult {
            data,
            offset: data_span.start,
            pattern,
            splits: pattern_splits,
            matched: true,
        };
    }

    matches.iter().skip(1).for_each(|&end| {
        let split = SearchSplit::new(
            span(data_span.start + start,
            data_span.start + end),
            data,
            pattern,
            1,
        );
        pattern_splits.push(split);
        start = end + pattern_len;
    });

    if start < data_span.len() - 1 {
        let split = SearchSplit::new(span(data_span.start + start, data_span.end), data, pattern, 0);
        pattern_splits.push(split);
    }

    SearchResult {
        data,
        offset: data_span.start,
        pattern,
        splits: pattern_splits,
        matched: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span;

    #[test]
    fn span_bounds_test() -> anyhow::Result<()> {
        let data = "Hello, you all! How are you?";
        assert_eq!(data.len(), 28);
        let data_span = Span {
            start: 0,
            end: data.len(),
        };
        assert!(span_within_bounds(data_span, data.len()));

        let start_empty_span = Span { start: 0, end: 0 };
        assert!(span_within_bounds(start_empty_span, data.len()));

        let valid_span = Span { start: 5, end: 10 };

        assert!(span_within_bounds(valid_span, data.len()));

        let empty_span = Span { start: 5, end: 5 };
        assert!(span_within_bounds(empty_span, data.len()));

        let wrong_span = Span { start: 5, end: 2 };
        assert!(!span_within_bounds(wrong_span, data.len()));

        let end_empty_span = Span {
            start: data.len(),
            end: data.len(),
        };
        assert!(!span_within_bounds(end_empty_span, data.len()));

        let end_wrong_span = Span {
            start: data.len() - 3,
            end: data.len() + 1,
        };

        assert!(!span_within_bounds(end_wrong_span, data.len()));

        Ok(())
    }

    #[test]
    fn span_to_string_test() -> anyhow::Result<()> {
        let data = "Hello, you all! How are you?";
        let empty_span = Span { start: 5, end: 5 };
        let empty_str = span_to_string(data, empty_span);
        assert_eq!(empty_str, Some("".to_string()));

        let valid_span = Span { start: 5, end: 10 };
        let valid_str = span_to_string(data, valid_span);
        assert_eq!(valid_str, Some(", you".to_string()));

        let data_span = Span {
            start: 0,
            end: data.len(),
        };
        let data_str = span_to_string(data, data_span);
        assert_eq!(data_str, Some(data.to_string()));

        let wrong_span = Span { start: 5, end: 2 };
        let wrong_str = span_to_string(data, wrong_span);
        assert_eq!(wrong_str, None);

        let end_span = Span {
            start: data.len() - 3,
            end: data.len() + 1,
        };
        let end_str = span_to_string(data, end_span);
        assert_eq!(end_str, None);

        Ok(())
    }

    #[test]
    fn no_matches_test() -> anyhow::Result<()> {
        let pattern = "\n\n";
        let data1 = "Hello, you all! How are you?";

        let no_match_res = find_pattern(pattern, data1.as_bytes(), span(0, data1.len()));
        assert!(!no_match_res.matched);

        assert_eq!(no_match_res.data()[0], data1.to_string());
        Ok(())
    }

    #[test]
    fn empty_data_test() -> anyhow::Result<()> {
        let pattern = "\n\n";
        let data1 = "";

        let result = find_pattern(pattern, data1.as_bytes(), span(0, data1.len()));
        assert!(!result.matched);

        Ok(())
    }

    #[test]
    fn single_ends_test() -> anyhow::Result<()> {
        let pattern = "\n\n";
        let data1 = "\n\nHello, you all! How are you?".as_bytes();
        let result1 = find_pattern(pattern, data1, span(0, data1.len()));

        assert_eq!(result1.splits.len(), 2);
        assert_eq!(result1.splits[0].span, Span { start: 0, end: 0 });
        assert_eq!(result1.splits[0].stride, 1);
        assert_eq!(result1.splits[0].data(), "".to_string());

        assert_eq!(result1.splits[1].span.len(), data1.len() - 2);
        assert_eq!(result1.splits[1].stride, 0);
        assert_eq!(
            result1.splits[1].data(),
            "Hello, you all! How are you?".to_string()
        );

        let data2 = "Hello, you all! How are you?\n\n".as_bytes();
        let result2 = find_pattern(pattern, data2, span(0, data2.len()));
        assert_eq!(result2.splits.len(), 1);
        assert_eq!(result2.splits[0].span.len(), data2.len() - 2);
        assert_eq!(result2.splits[0].stride, 1);
        assert_eq!(
            result2.splits[0].data(),
            "Hello, you all! How are you?".to_string()
        );

        Ok(())
    }

    #[test]
    fn single_middle_test() -> anyhow::Result<()> {
        let pattern = "\n\n";

        let data4 = "Hello, you all!\n\nHow are you ?".as_bytes();
        let result4 = find_pattern(pattern, data4, span(0, data4.len()));
        assert_eq!(result4.splits.len(), 2);
        assert_eq!(result4.splits[0].stride, 1);
        assert_eq!(result4.splits[1].stride, 0);
        assert_eq!(result4.splits[0].data(), "Hello, you all!".to_string());
        assert_eq!(result4.splits[1].data(), "How are you ?".to_string());

        Ok(())
    }

    #[test]
    fn find_multiple_patterns_ends_test() -> anyhow::Result<()> {
        let pattern = "\n\n";
        let data1 = "\n\nHello, you all! How are you? \n\n".as_bytes();
        let result1 = find_pattern(pattern, data1, span(0, data1.len()));

        assert_eq!(result1.splits.len(), 2);
        assert_eq!(result1.splits[0].stride, 1);
        assert_eq!(result1.splits[0].data(), "".to_string());
        assert_eq!(result1.splits[1].stride, 1);
        assert_eq!(
            result1.splits[1].data(),
            "Hello, you all! How are you? ".to_string()
        );

        let data2 = "\n\n\n\nHello, you all! How are you? \n\n\n\n".as_bytes();
        let result2 = find_pattern(pattern, data2, span(0, data2.len()));
        assert_eq!(result2.splits.len(), 4);
        assert_eq!(result2.splits[0].stride, 1);
        assert_eq!(result2.splits[0].data(), "".to_string());
        assert_eq!(result2.splits[1].stride, 1);
        assert_eq!(result2.splits[1].data(), "".to_string());
        assert_eq!(result2.splits[2].stride, 1);
        assert_eq!(
            result2.splits[2].data(),
            "Hello, you all! How are you? ".to_string()
        );
        assert_eq!(result2.splits[3].stride, 1);
        assert_eq!(result2.splits[3].data(), "".to_string());
        Ok(())
    }

    #[test]
    fn find_multiple_patterns_test() -> anyhow::Result<()> {
        let pattern = "\n\n";
        let data1 = "Hello, you all!\n\n How are you? \n\n".as_bytes();

        let result1 = find_pattern(pattern, data1, span(0, data1.len()));
        assert_eq!(result1.splits.len(), 2);
        assert_eq!(result1.splits[0].stride, 1);
        assert_eq!(result1.splits[0].data(), "Hello, you all!".to_string());
        assert_eq!(result1.splits[1].stride, 1);
        assert_eq!(result1.splits[1].data(), " How are you? ".to_string());

        let data2 = "Hello, you all!\n\n How are you? \n\n Nice to meet you all!".as_bytes();
        let result2 = find_pattern(pattern, data2, span(0, data2.len()));
        assert_eq!(result2.splits.len(), 3);
        assert_eq!(result2.splits[0].stride, 1);
        assert_eq!(result2.splits[0].data(), "Hello, you all!".to_string());
        assert_eq!(result2.splits[1].stride, 1);
        assert_eq!(result2.splits[1].data(), " How are you? ".to_string());
        assert_eq!(result2.splits[2].stride, 0);
        assert_eq!(
            result2.splits[2].data(),
            " Nice to meet you all!".to_string()
        );

        let data3 = "\n\nHello, you all!\n\n How are you? \n\n Nice to meet you all!\n\n I hope you are all doing well!".as_bytes();
        let result2 = find_pattern(pattern, data3, span(0, data3.len()));
        assert_eq!(result2.splits.len(), 5);
        assert_eq!(result2.splits[0].stride, 1);
        assert_eq!(result2.splits[0].data(), "".to_string());
        assert_eq!(result2.splits[1].stride, 1);
        assert_eq!(result2.splits[1].data(), "Hello, you all!".to_string());
        assert_eq!(result2.splits[2].stride, 1);
        assert_eq!(result2.splits[2].data(), " How are you? ".to_string());
        assert_eq!(result2.splits[3].stride, 1);
        assert_eq!(
            result2.splits[3].data(),
            " Nice to meet you all!".to_string()
        );
        assert_eq!(result2.splits[4].stride, 0);
        assert_eq!(
            result2.splits[4].data(),
            " I hope you are all doing well!".to_string()
        );

        println!("{:?}", result2);

        let data4 =
            "Hello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n".as_bytes();
        let result3 = find_pattern(pattern, data4, span(0, data4.len()));
        assert_eq!(result3.splits.len(), 4);
        assert_eq!(result3.splits[0].stride, 1);
        assert_eq!(result3.splits[0].data(), "Hello, you all!".to_string());
        assert_eq!(result3.splits[1].stride, 1);
        assert_eq!(result3.splits[1].data(), " How are you? ".to_string());
        assert_eq!(result3.splits[2].stride, 1);
        assert_eq!(result3.splits[2].data(), "".to_string());
        assert_eq!(result3.splits[3].stride, 1);
        assert_eq!(
            result3.splits[3].data(),
            " Nice to meet you all!".to_string()
        );

        let data5 =
            "\n\n\n\nHello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n\n\n"
                .as_bytes();
        let result4 = find_pattern(pattern, data5, span(0, data5.len()));
        assert_eq!(result4.splits.len(), 7);
        assert_eq!(result4.splits[0].stride, 1);
        assert_eq!(result4.splits[0].data(), "".to_string());
        assert_eq!(result4.splits[1].stride, 1);
        assert_eq!(result4.splits[1].data(), "".to_string());
        assert_eq!(result4.splits[2].stride, 1);
        assert_eq!(result4.splits[2].data(), "Hello, you all!".to_string());
        assert_eq!(result4.splits[3].stride, 1);
        assert_eq!(result4.splits[3].data(), " How are you? ".to_string());
        assert_eq!(result4.splits[4].stride, 1);
        assert_eq!(result4.splits[4].data(), "".to_string());
        assert_eq!(result4.splits[5].stride, 1);
        assert_eq!(
            result4.splits[5].data(),
            " Nice to meet you all!".to_string()
        );
        assert_eq!(result4.splits[6].stride, 1);
        assert_eq!(result4.splits[6].data(), "".to_string());

        Ok(())
    }

    #[test]
    fn reconstruct_test() -> anyhow::Result<()> {
        let pattern = "\n\n";
        let data1 = "Hello, you all!\n\n How are you? \n\n";
        let result1 = find_pattern(pattern, data1.as_bytes(), span(0, data1.len()));
        let reconsted1 = result1.reconstruct();
        assert_eq!(reconsted1, data1);

        let data2 = "Hello, you all!\n\n How are you? \n\n Nice to meet you all!";
        let result2 = find_pattern(pattern, data2.as_bytes(), span(0, data2.len()));
        let reconsted2 = result2.reconstruct();
        assert_eq!(reconsted2, data2);

        let data3 =
            "\n\n\n\nHello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n\n\n";

        let result3 = find_pattern(pattern, data3.as_bytes(), span(0, data3.len()));
        let reconsted3 = result3.reconstruct();
        assert_eq!(reconsted3, data3);

        let data4 = "\n\nHello, you all! How are you?";
        let result4 = find_pattern(pattern, data4.as_bytes(), span(0, data4.len()));
        let reconsted4 = result4.reconstruct();
        assert_eq!(reconsted4, data4);

        let data5 = "Hello, you all! How are you?";
        let result5 = find_pattern(pattern, data5.as_bytes(), span(0, data5.len()));
        let reconsted5 = result5.reconstruct();
        assert_eq!(reconsted5, data5);

        let data6 = "Hello, you all! How are you?\n\n";
        let result6 = find_pattern(pattern, data6.as_bytes(), span(0, data6.len()));
        let reconsted6 = result6.reconstruct();
        assert_eq!(reconsted6, data6);

        Ok(())
    }
}
