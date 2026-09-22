use crate::common::tokens_result_lite::TokensResultLite;
use crate::encodings::{EncodingType, Tokenize};
use aho_corasick::Span;
use unicode_categories::UnicodeCategories;

pub struct WSTokenizer {
    pub ascii: bool,
}

pub struct WSEncoding {
    pub offsets: Vec<(usize, usize)>,
}

impl Tokenize for WSTokenizer {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType> {
        let tokens_offsets = self.whitespace_punctuation_tokenize(data);
        Ok(EncodingType::WSEncoding(WSEncoding {
            offsets: tokens_offsets,
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
    pub fn divide_encoding_lite(&self, tokens_span: Option<Span>) -> TokensResultLite {
        if let Some(tokens_span) = tokens_span {
            let offsets = &self.offsets[tokens_span.range()];

            TokensResultLite {
                ids: None,
                offsets: Some(offsets.to_vec()),
            }
        } else {
            TokensResultLite {
                ids: None,
                offsets: None,
            }
        }
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
    use crate::normalizer::TextNormalizer;

    fn tokens<'a>(tokenizer: &WSTokenizer, text: &'a str) -> Vec<&'a str> {
        tokenizer
            .whitespace_punctuation_tokenize(text)
            .into_iter()
            .map(|(start, end)| &text[start..end])
            .collect()
    }

    #[test]
    fn ascii_tokens_match_reference_implementation() {
        let tokenizer = WSTokenizer { ascii: true };
        let greeting = " Hello, you all! How are you ? I am fine. Nice to meet you all insecure! ";
        assert_eq!(tokens(&tokenizer, greeting).len(), 20);

        let superlinear = "\"Seek competition\" is similarly useless; what if the prize isn't worth competing for? Sufficiently fast exponential growth guarantees both the shape and magnitude of the return curve — because something that grows fast enough will grow big even if it's trivially small at first — but thresholds only guarantee the shape. ";
        for text in [greeting, superlinear] {
            assert_eq!(tokens(&tokenizer, text), ws_punc_tokens(text));
        }
    }

    #[test]
    fn whitespace_only_input_has_no_tokens() {
        let tokenizer = WSTokenizer { ascii: false };
        for text in ["", " ", "  \n \t "] {
            assert!(tokens(&tokenizer, text).is_empty(), "{text:?}");
        }
    }

    /// Every token is a single punctuation mark or a run of word characters, and together
    /// the tokens hold every non-whitespace character exactly once.
    #[test]
    fn unicode_tokens_split_on_whitespace_and_punctuation() {
        let tokenizer = WSTokenizer { ascii: false };
        let chinese = TextNormalizer::default()
            .normalize(&"历史地理学的起源至少可以追溯到我国最早的地理学著作《山海经》".to_string())
            .unwrap();
        let texts = [
            "   \n \t This is a test, 123! :,+-     some more text. \n \n \t something else",
            "مرحبا، كيف حالك؟",
            "こんにちは、お元気ですか？",
            chinese.as_str(),
            "\"You get out,\" I heard a thousand times, \"what you put in.\" I'm not sure, I don't think so.",
        ];
        for text in texts {
            let mut previous_end = 0;
            let mut covered = 0;
            for (start, end) in tokenizer.whitespace_punctuation_tokenize(text) {
                assert!(
                    previous_end <= start && start < end && end <= text.len(),
                    "{text:?}: bad span ({start}, {end})"
                );
                assert!(text.is_char_boundary(start) && text.is_char_boundary(end));
                let token = &text[start..end];
                let mut chars = token.chars();
                let first = chars.next().unwrap();
                assert!(
                    !token.chars().any(|c| tokenizer.is_whitespace(&c)),
                    "{token:?} contains whitespace"
                );
                if tokenizer.is_punctuation(&first) {
                    assert_eq!(
                        chars.next(),
                        None,
                        "punctuation token {token:?} is not a single char"
                    );
                } else {
                    assert!(
                        !token.chars().any(|c| tokenizer.is_punctuation(&c)),
                        "{token:?} mixes word and punctuation characters"
                    );
                }
                covered += token.chars().count();
                previous_end = end;
            }
            let non_whitespace = text.chars().filter(|c| !tokenizer.is_whitespace(c)).count();
            assert_eq!(
                covered, non_whitespace,
                "{text:?}: tokens miss or repeat characters"
            );
        }
    }
}
