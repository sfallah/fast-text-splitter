use crate::common::tokens_result_lite::TokensResultLite;
use crate::encodings::{EncodingType, Tokenize};
use aho_corasick::Span;
use unicode_categories::UnicodeCategories;

pub struct WSTokenizer {
    pub ascii: bool,
}

pub struct WSEncoding {
    pub offsets: Vec<(usize, usize)>,
    pub word_ids: Vec<Option<u32>>,
}

impl Tokenize for WSTokenizer {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType> {
        let tokens_offsets = self.whitespace_punctuation_tokenize(data);
        let word_ids: Vec<_> = (0..tokens_offsets.len()).map(|x| Some(x as u32)).collect();
        Ok(EncodingType::WSEncoding(WSEncoding {
            offsets: tokens_offsets,
            word_ids,
        }))
    }
}

impl WSTokenizer {
    pub fn is_whitespace(&self, c: &char) -> bool {
        if self.ascii {
            c.is_ascii_whitespace()
        } else {
            c.is_whitespace()
        }
    }

    pub fn is_punctuation(&self, c: &char) -> bool {
        if self.ascii {
            c.is_ascii_punctuation()
        } else {
            c.is_punctuation()
        }
    }

    pub fn whitespace_punctuation_tokenize(&self, text: &str) -> Vec<(usize, usize)> {
        let mut offsets = Vec::new();
        let mut start_index = 0;
        let mut current_index = 0;
        let mut in_token = false;

        let mut chars = text.chars();

        while let Some(c) = chars.next() {
            let char_len = c.len_utf8();
            if self.is_whitespace(&c) {
                if in_token {
                    offsets.push((start_index, current_index));
                    in_token = false;
                }
                current_index += char_len;
            } else if self.is_punctuation(&c) {
                if in_token {
                    offsets.push((start_index, current_index));
                    in_token = false;
                }
                offsets.push((current_index, current_index + char_len));
                current_index += char_len;
            } else {
                if !in_token {
                    start_index = current_index;
                    in_token = true;
                }
                current_index += char_len;
            }
        }

        if in_token {
            offsets.push((start_index, current_index));
        }

        offsets
    }
}

impl WSEncoding {
    pub fn divide_encoding_lite(&self, tokens_span: Span) -> TokensResultLite {
        let result = {
            let offsets = &self.offsets[tokens_span.range()];

            TokensResultLite {
                ids: None,
                offsets: Some(offsets.to_vec()),
            }
        };
        result
    }
}

pub fn ws_punc_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    for c in text.chars() {
        if c.is_whitespace() {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        } else if c.is_ascii_punctuation() {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
            tokens.push(c.to_string());
        } else {
            current_token.push(c);
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[test]
    fn alphanumeric_test() {
        assert!(!'.'.is_alphanumeric());
        assert!(!'!'.is_alphanumeric());
        assert!(!' '.is_alphanumeric());
        assert!(!'?'.is_alphanumeric());
        assert!(!'('.is_alphanumeric());
        assert!(!')'.is_alphanumeric());
        assert!(!'['.is_alphanumeric());
        assert!(!']'.is_alphanumeric());
        assert!(!'{'.is_alphanumeric());
        assert!(!'}'.is_alphanumeric());
        assert!(!'@'.is_alphanumeric());
        assert!(!'#'.is_alphanumeric());
        assert!(!'$'.is_alphanumeric());
        assert!(!'%'.is_alphanumeric());
    }

    fn get_token(data: &str, start: usize, end: usize) -> String {
        from_utf8(&data.as_bytes()[start..end]).unwrap().to_string()
    }

    #[test]
    fn token_span_test() {
        let ws_tokenizer = WSTokenizer { ascii: false };
        let en_data =
            "   \n \t This is a test, 123! :,+-     some more text. \n \n \t something else";

        println!("{:?}", en_data);
        let en_tokens = ws_tokenizer.whitespace_punctuation_tokenize(en_data);
        println!("{:?}", en_tokens);

        en_tokens.iter().for_each(|(start, end)| {
            let token = get_token(en_data, *start, *end);
            println!("{}", token);
        });

        let ar_data = "مرحبا، كيف حالك؟";
        let ar_tokens = ws_tokenizer.whitespace_punctuation_tokenize(ar_data);

        println!("{:?}", ar_tokens);
        ar_tokens.iter().for_each(|(start, end)| {
            let token = get_token(ar_data, *start, *end);
            println!("{}", token);
        });

        let ja_data = "こんにちは、お元気ですか？";
        let ja_tokens = ws_tokenizer.whitespace_punctuation_tokenize(ja_data);
        ja_tokens.iter().for_each(|(start, end)| {
            let token = get_token(ja_data, *start, *end);
            println!("{}", token);
        });

        let df_data = "\"You get out,\" I heard a thousand times, \"what you put in.\" I'm not sure, I don't think so.";

        let df_tokens = ws_tokenizer.whitespace_punctuation_tokenize(df_data);
        println!("tokens_no: {:?}", df_tokens.len());

        df_tokens.iter().for_each(|(start, end)| {
            let token = get_token(df_data, *start, *end);
            println!("{:?}", token);
        });
    }

    #[test]
    fn ws_test() {
        let ws_tokenizer = WSTokenizer { ascii: true };
        let data = " Hello, you all! How are you ? I am fine. Nice to meet you all insecure! ";
        let wd_spans = ws_tokenizer.whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 20);
        let mut ws_tokens = Vec::new();
        for (start, end) in wd_spans.iter() {
            let token = get_token(data, *start, *end);
            ws_tokens.push(token.clone());
            println!("'{}'", token);
        }

        let expected_tokens = ws_punc_tokens(data);
        assert_eq!(ws_tokens, expected_tokens);
    }

    #[test]
    fn ws_empty_test() {
        let ws_tokenizer = WSTokenizer { ascii: false };

        let data = "";
        let wd_spans = ws_tokenizer.whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 0);

        let data = " ";
        let wd_spans = ws_tokenizer.whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 0);

        let data = "  \n \t ";
        let wd_spans = ws_tokenizer.whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 0);
    }

    #[test]
    fn ws_superlinear_split_test() {
        let ws_tokenizer = WSTokenizer { ascii: true };

        let split_string = "\"Seek competition\" is similarly useless; what if the prize isn't worth competing for? Sufficiently fast exponential growth guarantees both the shape and magnitude of the return curve — because something that grows fast enough will grow big even if it's trivially small at first — but thresholds only guarantee the shape. ";

        let tokens_spans = ws_tokenizer.whitespace_punctuation_tokenize(split_string);

        let ws_tokens: Vec<_> = tokens_spans
            .iter()
            .map(|(start, end)| get_token(split_string, *start, *end))
            .collect();
        println!("{:?}", ws_tokens);
        let tokens = ws_punc_tokens(split_string);
        println!("{:?}", tokens);
        assert_eq!(tokens.len(), tokens_spans.len());
        assert_eq!(tokens, ws_tokens);

        let problem_chars = split_string
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_alphanumeric() && !c.is_ascii_punctuation())
            .map(|c| format!("'{}'", c))
            .collect::<String>();
        println!("{:?}", problem_chars);
    }
}
