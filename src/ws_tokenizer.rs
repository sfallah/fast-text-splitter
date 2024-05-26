use super::Tokenize;
use crate::encodings::EncodingType;

pub struct WSTokenizer;

pub struct WSEncoding {
    pub offsets: Vec<(usize, usize)>,
    pub word_ids: Vec<Option<u32>>,
}

impl Tokenize for WSTokenizer {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType> {
        let tokens_offsets = whitespace_punctuation_tokenize(data);
        let word_ids: Vec<_> = (0..tokens_offsets.len()).map(|x| Some(x as u32)).collect();
        Ok(EncodingType::WSEncoding(WSEncoding {
            offsets: tokens_offsets,
            word_ids,
        }))
    }
}

pub fn whitespace_punctuation_tokenize(data: &str) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = vec![];
    let mut start: usize = 0;
    let mut bytes_start: usize = 0;
    let mut bytes_end: usize = 0;
    data.chars().enumerate().for_each(|(idx, c)| {
        let c_len = c.len_utf8();
        if !c.is_alphanumeric() {
            if start < idx {
                spans.push((bytes_start, bytes_end));
            }
            if c.is_ascii_punctuation() {
                spans.push((bytes_end, bytes_end + c_len));
            }
            start = idx + 1;
            bytes_start = bytes_end + c_len;
        }
        bytes_end += c_len;
    });
    if bytes_start < data.len() {
        spans.push((bytes_start, data.len()));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws_punc_tokens(text: &str) -> Vec<String> {
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

    fn tokenize_with_offsets_zero(text: &str) -> Vec<(usize, usize)> {
        let mut offsets = Vec::new();
        let mut start_index = 0;
        let mut current_index = 0;
        let mut in_token = false;

        let mut chars = text.chars();

        while let Some(c) = chars.next() {
            let char_len = c.len_utf8();
            if c.is_whitespace() {
                if in_token {
                    offsets.push((start_index, current_index));
                    in_token = false;
                }
                current_index += char_len;
            } else if c.is_ascii_punctuation() {
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
        data[start..end].to_string()
    }

    #[test]
    fn token_span_test() {
        let en_data =
            "   \n \t This is a test, 123! :,+-     some more text. \n \n \t something else";

        println!("{:?}", en_data);
        let en_tokens = whitespace_punctuation_tokenize(en_data);
        println!("{:?}", en_tokens);

        en_tokens.iter().for_each(|(start, end)| {
            let token = get_token(en_data, *start, *end);
            println!("{}", token);
        });

        let ar_data = "مرحبا، كيف حالك؟";
        let ar_tokens = whitespace_punctuation_tokenize(ar_data);

        println!("{:?}", ar_tokens);
        ar_tokens.iter().for_each(|(start, end)| {
            let token = get_token(ar_data, *start, *end);
            println!("{}", token);
        });

        let ja_data = "こんにちは、お元気ですか？";
        let ja_tokens = whitespace_punctuation_tokenize(ja_data);
        ja_tokens.iter().for_each(|(start, end)| {
            let token = get_token(ja_data, *start, *end);
            println!("{}", token);
        });

        let df_data = "\"You get out,\" I heard a thousand times, \"what you put in.\" I'm not sure, I don't think so.";

        let df_tokens = whitespace_punctuation_tokenize(df_data);
        println!("tokens_no: {:?}", df_tokens.len());

        df_tokens.iter().for_each(|(start, end)| {
            let token = get_token(df_data, *start, *end);
            println!("{:?}", token);
        });
    }

    #[test]
    fn ws_test() {
        let data = " Hello, you all! How are you ? I am fine. Nice to meet you all insecure! ";
        let wd_spans = whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 20);
        let mut ws_tokens = Vec::new();
        for (start, end) in wd_spans.iter() {
            let token = get_token(data, *start, *end);
            ws_tokens.push(token.clone());
            println!("'{}'", token);
        }

        let zero_cp_offsets = tokenize_with_offsets_zero(data);
        assert_eq!(wd_spans, zero_cp_offsets);

        let expected_tokens = ws_punc_tokens(data);
        assert_eq!(ws_tokens, expected_tokens);
    }

    #[test]
    fn ws_empty_test() {
        let data = "";
        let wd_spans = whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 0);

        let data = " ";
        let wd_spans = whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 0);

        let data = "  \n \t ";
        let wd_spans = whitespace_punctuation_tokenize(data);
        assert_eq!(wd_spans.len(), 0);
    }
}
