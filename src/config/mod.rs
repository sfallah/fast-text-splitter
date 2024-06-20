use aho_corasick::Span;

use crate::encodings::Tokenize;

use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
use crate::pattern_search::pattern_searcher::PatternSearcher;
use crate::splitter::split_node::utils::SplitResultLite;
use crate::splitter::Splitter;
use crate::ws_tokenizer::WSTokenizer;

pub struct SplitterLiteConfig<T: Tokenize + Sync> {
    pub patterns: Vec<Vec<String>>,
    pub searchers: Vec<PatternSearcher>,
    pub tokenizer: T,
    pub max_tokens: Option<usize>,
    pub merge_level: Option<usize>,
    pub parallel: Option<bool>,
}

impl SplitterLiteConfig<WSTokenizer> {
    pub fn new_ws(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
    ) -> Self {
        let searchers: Vec<_> = patterns.clone().iter().map(|p| PatternSearcher::new(p.clone())).collect();
        Self {
            patterns: patterns.clone(),
            searchers,
            tokenizer: WSTokenizer {},
            max_tokens: Some(max_tokens),
            merge_level: Some(merge_level),
            parallel: Some(parallel),
        }
    }

    pub fn ws_splits(&self, data: &[u8]) -> Vec<SplitResultLite> {
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = crate::splitter::splitter_config::SplitterConfig::<WSTokenizer> {
            data,
            patterns: self.patterns.clone(),
            searchers: &self.searchers,
            max_len: self.max_tokens,
            tokenizer: Some(&self.tokenizer),
        };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();
        tree.get_results_lite(self.max_tokens, data)
    }
}

impl SplitterLiteConfig<HFTokenizer> {
    pub fn new_hf(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
        model: Option<String>,
    ) -> Self {
        let searchers: Vec<_> = patterns.clone().iter().map(|p| PatternSearcher::new(p.clone())).collect();
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(model, Some(usize::MAX), false).unwrap(),
        };
        Self {
            patterns: patterns.clone(),
            searchers,
            tokenizer: hf_tokenizer,
            max_tokens: Some(max_tokens),
            merge_level: Some(merge_level),
            parallel: Some(parallel),
        }
    }

    pub fn hf_splits(&self, data: &[u8]) -> Vec<SplitResultLite> {
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = crate::splitter::splitter_config::SplitterConfig::<HFTokenizer> {
            data,
            patterns: self.patterns.clone(),
            searchers: &self.searchers,
            max_len: self.max_tokens,
            tokenizer: Some(&self.tokenizer),
        };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();
        tree.get_results_lite(self.max_tokens, data)
    }
}
