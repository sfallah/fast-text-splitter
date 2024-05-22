use aho_corasick::{Span};
use memchr::memmem::find_iter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchSplit {
    pub dt_start: usize,
    pub dt_end: usize,
    pub mt_pattern_id: Option<usize>,
    pub sub_splits: Option<Vec<MatchSplit>>,
}

pub fn find_matches(pattern: &str, data: &str) -> Vec<Span> {
    let pattern_len = pattern.len();
    let matches: Vec<_> = find_iter(data.as_bytes(), pattern.as_bytes()).collect();
    let mut splits = Vec::new();
    if matches.is_empty() {
        splits.push(Span { start: 0, end: data.len() });
        return splits;
    }

    let mut start = matches[0];

    if start > 0 {
        splits.push(Span { start: 0, end: start + pattern_len});
        start += pattern_len;
    } else {
        if matches.len() == 1 {
            splits.push(Span { start: 0, end: data.len() });
            return splits;
        }
    }

    matches.iter().skip(1).for_each(|&end| {
        splits.push(Span { start, end: end + pattern_len});
        start = end + pattern_len;
    });
    if start < data.len() {
        splits.push(Span { start, end: data.len() });
    }
    splits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_spans(spans: &Vec<Span>, data: &str) {
        println!("{:?}", data);
        for span in spans.iter() {
            println!("{:?}", span);
            println!("{:?}", &data[span.start..span.end]);
        }
    }

    #[test]
    fn find_matches_test() -> anyhow::Result<()> {

        let pattern = "\n\n";
        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

        let spans = find_matches("\n\n", data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";

        let spans = find_matches("\n\n", data);
        print_spans(&spans, data);

        let data = "Hello, you all! How are you ? \n\n I am fine. Nice to meet you all insecure!";
        let spans = find_matches("\n\n", data);
        print_spans(&spans, data);


        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure! \n\n";
        let spans = find_matches("\n\n", data);
        print_spans(&spans, data);


        let data = "\n\n Hello, you all! How are you ? \n I am fine. \n\n Nice to meet you all insecure! \n And it continues!";
        let spans = find_matches("\n\n", data);
        print_spans(&spans, data);

        Ok(())
    }
}
