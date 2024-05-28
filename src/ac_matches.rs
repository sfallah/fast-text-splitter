use aho_corasick::{AhoCorasick, MatchKind, Span};
use memchr::memmem::find_iter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub splits: Vec<Span>,
    pub matched: bool,
}

pub fn merge_partition_spans(splits: &Vec<Span>, part_size: usize) -> Vec<Span> {
    splits
        .chunks(part_size)
        .map(|chunk| {
            let start = chunk[0].start;
            let end = chunk[chunk.len() - 1].end;
            Span { start, end }
        })
        .collect()
}

pub fn init_aho_corasick(patterns: &Vec<String>) -> anyhow::Result<AhoCorasick> {
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)?;
    Ok(ac)
}

pub fn find_patterns_matches(patterns: &Vec<String>, data_bytes: &[u8]) -> MatchResult {
    if patterns.len() == 1 {
        return find_pattern_matches(&patterns[0], data_bytes);
    }
    let ac = init_aho_corasick(patterns).unwrap();
    let matches: Vec<_> = ac.find_iter(data_bytes).collect();
    let mut splits = Vec::new();
    if matches.is_empty() {
        splits.push(Span {
            start: 0,
            end: data_bytes.len(),
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
                end: data_bytes.len(),
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
    if start < data_bytes.len() {
        splits.push(Span {
            start,
            end: data_bytes.len(),
        });
    }
    MatchResult {
        splits,
        matched: true,
    }
}

pub fn find_pattern_matches(pattern: &str, data_bytes: &[u8]) -> MatchResult {
    let pattern_len = pattern.len();
    let matches: Vec<_> = find_iter(data_bytes, pattern.as_bytes()).collect();
    let mut splits = Vec::new();
    if matches.is_empty() {
        splits.push(Span {
            start: 0,
            end: data_bytes.len(),
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
                end: data_bytes.len(),
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
    if start < data_bytes.len() {
        splits.push(Span {
            start,
            end: data_bytes.len(),
        });
    }
    MatchResult {
        splits,
        matched: true,
    }
}

#[cfg(test)]
mod tests {
    use std::string::String;
    use crate::hf_tokenizer::init_tokenizer;
    use crate::normalizer::TextNormalizer;
    use super::*;

    fn print_spans(spans: &MatchResult, data: &[u8]) {
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
        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!".as_bytes();
        let patterns = vec!["\n\n".to_string()];

        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? I am fine. Nice to meet you all insecure!".as_bytes();

        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "Hello, you all! How are you ? \n\n I am fine. Nice to meet you all insecure!".as_bytes();
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure! \n\n".as_bytes();
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? \n I am fine. \n\n Nice to meet you all insecure! \n And it continues!".as_bytes();
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let data = "\n\n Hello, you all! How are you ? \n I am fine. Nice to meet you all insecure! \n \n \n\n".as_bytes();
        let spans = find_patterns_matches(&patterns, data);
        print_spans(&spans, data);

        let second_patterns = vec!["\n".to_string()];
        let spans = find_patterns_matches(&second_patterns, data);
        print_spans(&spans, data);

        Ok(())
    }

    #[test]
    fn not_normalized_test() -> anyhow::Result<()>{
        let normalizer = TextNormalizer::new(true, false, None, false);

        let data_file = "tests/error_data/Selena Gomez - Wikipedia.txt";
        let data= std::fs::read_to_string(data_file)?;


        //let binding = normalizer.normalize(&data)?;
        //let data_bytes = binding.as_bytes();
        let data_bytes = data.as_bytes();

        let match_result = find_patterns_matches(&vec!["\n\n".to_string()], data_bytes);
        println!("Matched: {:?}", match_result.splits.len());

        let mut reconstructed_data = String::new();
        for span in match_result.splits.iter() {
            reconstructed_data.push_str(std::str::from_utf8(&data_bytes[span.start..span.end])?);
        }
        assert_eq!(reconstructed_data, data);

        let tokenizer = init_tokenizer(None, Some(data.len())).unwrap();
        let once_encoded = tokenizer.encode(data.as_str(), false).unwrap();
        println!("Once Tokens: {:?}", once_encoded.len());

        let mut total_tokens: usize = 0;

        for span in match_result.splits.iter() {
            let sub_data = std::str::from_utf8(&data_bytes[span.start..span.end])?;
            let encoded_res = tokenizer.encode(sub_data, false);
            match encoded_res {
                Ok(encoded) => {
                    total_tokens += encoded.len();
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    println!("Data Span: {:?}", span);
                }
            }
        }
        println!("Total Tokens: {:?}", total_tokens);
        assert_eq!(total_tokens, once_encoded.len());

        let normalized_data = normalizer.normalize(&data.to_string())?;

        let normalized_encoded = tokenizer.encode(normalized_data.as_str(), false).unwrap();
        println!("Normalized Tokens: {:?}", normalized_encoded.len());
        assert_eq!(normalized_encoded.len(), once_encoded.len());


        let nomalized_match_result = find_patterns_matches(&vec!["\n\n".to_string()], normalized_data.as_bytes());
        println!("Normalized Matched: {:?}", nomalized_match_result.splits.len());

        let mut normalized_total_tokens: usize = 0;

        for span in nomalized_match_result.splits.iter() {
            let sub_data = std::str::from_utf8(&normalized_data.as_bytes()[span.start..span.end])?;
            let encoded_res = tokenizer.encode(sub_data, false);
            match encoded_res {
                Ok(encoded) => {
                    normalized_total_tokens += encoded.len();
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    println!("Data Span: {:?}", span);
                }
            }
        }
        println!("Normalized Total Tokens: {:?}", normalized_total_tokens);

        Ok(())
    }
}
