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

pub fn ws_spans(ws_idxs: Vec<usize>) -> Vec<Span> {
    let mut spans: Vec<Span> = vec![];
    let mut start: usize = ws_idxs[0];
    let mut end: usize = start;

    for idx in ws_idxs.iter().skip(1) {
        if *idx == end + 1 {
            end = *idx;
        } else {
            spans.push(Span {
                start,
                end: end + 1,
            });
            start = *idx;
            end = start;
        }
    }

    spans.push(Span {
        start,
        end: end + 1,
    });

    spans
}

pub fn word_spans(ws_spans: Vec<Span>, data_len: usize) -> Vec<Span> {
    let mut w_spans: Vec<Span> = vec![];
    let mut prev = ws_spans[0];
    let mut start: usize = prev.start + prev.len();
    if prev.start > 0 {
        w_spans.push(Span {
            start,
            end: prev.start,
        });
        start = prev.start + prev.len();
    }
    for span in ws_spans.iter().skip(1) {
        w_spans.push(Span {
            start,
            end: span.start,
        });
        start = span.start + span.len();
        prev = *span;
    }

    if prev.end < data_len {
        w_spans.push(Span {
            start,
            end: data_len,
        });
    }

    w_spans
}

pub fn word_tokenize(data: &str) -> Vec<(usize, usize)> {
    let indexes: Vec<_> = whitespace_indices(data);
    let spans = ws_spans(indexes);
    let word_spans = word_spans(spans, data.len());

    word_spans
        .iter()
        .map(|span| (span.start, span.end))
        .collect()
}

pub fn whitespace_indices(input: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            indices.push(i);
        }
    }
    indices
}
