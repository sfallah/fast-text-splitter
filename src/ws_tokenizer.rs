use super::Tokenize;
use crate::encodings::EncodingType;
use aho_corasick::Span;

pub struct WSTokenizer;

pub struct WSEncoding {
    pub offsets: Vec<(usize, usize)>,
    pub word_ids: Vec<Option<u32>>,
}

impl Tokenize for WSTokenizer {
    fn encode(&self, data: &str) -> anyhow::Result<EncodingType> {
        let tokens_offsets = word_tokenize(data);
        let word_ids: Vec<_> = (0..tokens_offsets.len()).map(|x| Some(x as u32)).collect();
        Ok(EncodingType::WSEncoding(WSEncoding {
            offsets: tokens_offsets,
            word_ids,
        }))
    }
}


pub fn word_tokenize(data: &str) -> Vec<(usize, usize)> {
    let mut spans: Vec<Span> = vec![];
    let mut start: usize = 0;
    let mut bytes_start: usize = 0;
    let mut bytes_end: usize = 0;
    data.chars().enumerate().for_each(|(idx, c)| {
        if !c.is_alphanumeric() {
            if start < idx {
                spans.push(Span {
                    start: bytes_start,
                    end: bytes_end,
                });
            }
            if !c.is_whitespace() {
                spans.push(Span {
                    start: bytes_end,
                    end: bytes_end + c.len_utf8(),
                });
            }
            start = idx + 1;
            bytes_start = bytes_end + c.len_utf8();
        }
        bytes_end += c.len_utf8();
    });
    if bytes_start < data.len() {
        spans.push(Span {
            start: bytes_start,
            end: data.len(),
        });
    }
    spans.iter().map(|span| (span.start, span.end)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let en_data = "   \n \t This is a test, 123! :,+-     some more text. \n \n \t something else";

        println!("{:?}", en_data);
        let en_tokens = word_tokenize(en_data);
        println!("{:?}", en_tokens);

        en_tokens.iter().for_each(|(start, end)| {
            let token = get_token(en_data, *start, *end);
            println!("{}", token);
        });

        let ar_data = "مرحبا، كيف حالك؟";
        let ar_tokens = word_tokenize(ar_data);

        println!("{:?}", ar_tokens);
        ar_tokens.iter().for_each(|(start, end)| {
            let token = get_token(ar_data, *start, *end);
            println!("{}", token);
        });

        let ja_data = "こんにちは、お元気ですか？";
        let ja_tokens = word_tokenize(ja_data);
        ja_tokens.iter().for_each(|(start, end)| {
            let token = get_token(ja_data, *start, *end);
            println!("{}", token);
        });

        let df_data = "\"You get out,\" I heard a thousand times, \"what you put in.\" I'm not sure, I don't think so.";

        let df_tokens = word_tokenize(df_data);
        println!("tokens_no: {:?}", df_tokens.len());

        df_tokens.iter().for_each(|(start, end)| {
            let token = get_token(df_data, *start, *end);
            println!("{:?}", token);
        });
    }

    #[test]
    fn ws_test() {
        let data = " Hello, you all! How are you ? I am fine. Nice to meet you all insecure! ";
        let wd_spans = word_tokenize(data);
        assert_eq!(wd_spans.len(), 20);
        for (start, end) in wd_spans.iter() {
            let token = get_token(data, *start, *end);
            println!("'{}'", token);
        }
    }

    #[test]
    fn ws_empty_test() {
        let data = "";
        let wd_spans = word_tokenize(data);
        assert_eq!(wd_spans.len(), 0);

        let data = " ";
        let wd_spans = word_tokenize(data);
        assert_eq!(wd_spans.len(), 0);

        let data = "  \n \t ";
        let wd_spans = word_tokenize(data);
        assert_eq!(wd_spans.len(), 0);
    }
}
