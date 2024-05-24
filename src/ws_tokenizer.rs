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
    data.chars().enumerate().for_each(|(idx, c)| {
        if !c.is_alphanumeric() {
            if start < idx {
                spans.push(Span {
                    start,
                    end: idx,
                });
            }
            if !c.is_whitespace() {
                spans.push(Span {
                    start: idx,
                    end: idx + 1,
                });
            }
            start = idx + 1;
        }
    });
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

    #[test]
    fn token_span_test() {
        let en_data = "   \n \t This is a test, 123! :,+-     some more text. \n \n \t something else";

        println!("{:?}", en_data);
        let en_tokens = word_tokenize(en_data);
        println!("{:?}", en_tokens);

        en_tokens.iter().for_each(|(start, end)| {
            let token = en_data.chars().skip(*start).take(*end - *start).collect::<String>();
            println!("{}", token);
        });

        let ar_data = "مرحبا، كيف حالك؟";
        let ar_tokens = word_tokenize(ar_data);

        println!("{:?}", ar_tokens);
        ar_tokens.iter().for_each(|(start, end)| {
            let token = ar_data.chars().skip(*start).take(*end - *start).collect::<String>();
            println!("{}", token);
        });

        let ja_data = "こんにちは、お元気ですか？";
        let ja_tokens = word_tokenize(ja_data);
        ja_tokens.iter().for_each(|(start, end)| {
            let token = ja_data.chars().skip(*start).take(*end - *start).collect::<String>();
            println!("{}", token);
        });

        let df_data = "\"You get out,\" I heard a thousand times, \"what you put in.\" I'm not sure, I don't think so.";

        let df_tokens = word_tokenize(df_data);
        println!("tokens_no: {:?}", df_tokens.len());

        df_tokens.iter().for_each(|(start, end)| {
            let token = df_data.chars().skip(*start).take(*end - *start).collect::<String>();
            println!("{:?}", token);
        });
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
