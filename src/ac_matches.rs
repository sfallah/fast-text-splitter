use aho_corasick::{AhoCorasick, MatchKind, Span};
use memchr::memmem::find_iter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub splits: Vec<Span>,
    pub matched: bool,
}

pub fn init_aho_corasick(patterns: &Vec<String>) -> anyhow::Result<AhoCorasick> {
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)?;
    Ok(ac)
}

pub fn find_patterns_matches(patterns: &Vec<String>, data: &str) -> MatchResult {
    if patterns.len() == 1 {
        return find_pattern_matches(&patterns[0], data);
    }
    let ac = init_aho_corasick(patterns).unwrap();
    let matches: Vec<_> = ac.find_iter(data).collect();
    let mut splits = Vec::new();
    if matches.is_empty() {
        splits.push(Span {
            start: 0,
            end: data.len(),
        });
        return MatchResult {
            splits,
            matched: false,
        };
    }

    let cur_match = matches[0];

    let mut start = cur_match.start();
    let mut pattern_len = cur_match.len();

    if start > 0 {
        splits.push(Span {
            start: 0,
            end: start + pattern_len,
        });
        start += pattern_len;
    } else {
        if matches.len() == 1 {
            splits.push(Span {
                start: 0,
                end: data.len(),
            });
            return MatchResult {
                splits,
                matched: true,
            };
        }
    }

    matches.iter().skip(1).for_each(|&mat| {
        let end = mat.start();
        pattern_len = mat.len();
        splits.push(Span {
            start,
            end: end + pattern_len,
        });
        start = end + pattern_len;
    });
    if start < data.len() {
        splits.push(Span {
            start,
            end: data.len(),
        });
    }
    MatchResult {
        splits,
        matched: true,
    }
}

pub fn find_pattern_matches(pattern: &str, data: &str) -> MatchResult {
    let pattern_len = pattern.len();
    let matches: Vec<_> = find_iter(data.as_bytes(), pattern.as_bytes()).collect();
    let mut splits = Vec::new();
    if matches.is_empty() {
        splits.push(Span {
            start: 0,
            end: data.len(),
        });
        return MatchResult {
            splits,
            matched: false,
        };
    }

    let mut start = matches[0];

    if start > 0 {
        splits.push(Span {
            start: 0,
            end: start + pattern_len,
        });
        start += pattern_len;
    } else {
        if matches.len() == 1 {
            splits.push(Span {
                start: 0,
                end: data.len(),
            });
            return MatchResult {
                splits,
                matched: true,
            };
        }
    }

    matches.iter().skip(1).for_each(|&end| {
        splits.push(Span {
            start,
            end: end + pattern_len,
        });
        start = end + pattern_len;
    });
    if start < data.len() {
        splits.push(Span {
            start,
            end: data.len(),
        });
    }
    MatchResult {
        splits,
        matched: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_spans(spans: &MatchResult, data: &str) {
        println!("-------------------");
        println!("Input Data: {:?}", data);
        for span in spans.splits.iter() {
            println!("{:?}", span);
            println!("{:?}", &data[span.start..span.end]);
            println!("-------------------");
        }
    }

    #[test]
    fn find_matches_test() -> anyhow::Result<()> {
        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";
        let patterns = vec!["\n\n".to_string()];

        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "Hello, you all! How are you ? \n\n I am fine. Nice to meet you all insecure!";
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure! \n\n";
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? \n I am fine. \n\n Nice to meet you all insecure! \n And it continues!";
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? \n I am fine. Nice to meet you all insecure! \n \n \n\n";
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let second_patterns = vec!["\n".to_string()];
        let spans = find_patterns_matches(&second_patterns, data);
        print_spans(&spans, data);

        Ok(())
    }
}
