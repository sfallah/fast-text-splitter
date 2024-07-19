use crate::common::split::Split;
use crate::encodings::{NoneTokenizer, Tokenize};
use aho_corasick::Span;

use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
use crate::pattern_search::pattern_searcher::PatternSearcher;
use crate::splitter::split_node::utils::SplitResultLite;
use crate::splitter::split_node::visualization::term_tree;
use crate::splitter::Splitter;
use crate::ws_tokenizer::WSTokenizer;

pub struct SplitterLiteConfig<T: Tokenize + Sync> {
    pub searchers: Vec<PatternSearcher>,
    pub tokenizer: T,
    pub max_tokens: Option<usize>,
    pub merge_level: Option<usize>,
    pub parallel: Option<bool>,
    pub patterns_len: usize,
}

impl SplitterLiteConfig<NoneTokenizer> {
    pub fn new_none(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
    ) -> Self {
        let patterns_len = patterns.len();
        let searchers: Vec<_> = patterns
            .clone()
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        Self {
            searchers,
            tokenizer: NoneTokenizer {},
            max_tokens: Some(max_tokens),
            merge_level: Some(merge_level),
            parallel: Some(parallel),
            patterns_len,
        }
    }

    //FIXME: max_tokens is not used needs a fix deeper in
    pub fn len_splits(&self, data: &[u8]) -> Vec<SplitResultLite> {
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = crate::splitter::splitter_config::SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &self.searchers,
            max_len: self.max_tokens,
            merge_level: self.merge_level,
            //max_len: None,
            tokenizer: None,
            patterns_len: self.patterns_len,
        };
        //FIXME: parallel is not used id fixed to true
        let splitter = Splitter::new(&config, span, 0, span, self.parallel, None, None);
        let tree = splitter.split();
        tree.get_results_lite(self.max_tokens, self.merge_level, data)
        //tree.get_results_lite(None, data)
    }
}

impl SplitterLiteConfig<WSTokenizer> {
    pub fn new_ws(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
        ascii: bool,
    ) -> Self {
        let patterns_len = patterns.len();
        let searchers: Vec<_> = patterns
            .clone()
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        Self {
            searchers,
            tokenizer: WSTokenizer { ascii },
            max_tokens: Some(max_tokens),
            merge_level: Some(merge_level),
            parallel: Some(parallel),
            patterns_len,
        }
    }

    pub fn ws_splits(&self, data: &[u8]) -> Vec<SplitResultLite> {
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = crate::splitter::splitter_config::SplitterConfig::<WSTokenizer> {
            data,
            searchers: &self.searchers,
            max_len: self.max_tokens,
            merge_level: self.merge_level,
            tokenizer: Some(&self.tokenizer),
            patterns_len: self.patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, self.parallel, None, None);
        let tree = splitter.split();
        tree.get_results_lite(self.max_tokens, self.merge_level, data)
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
        let patterns_len = patterns.len();
        let searchers: Vec<_> = patterns
            .clone()
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(model, Some(usize::MAX), false).unwrap(),
        };
        Self {
            searchers,
            tokenizer: hf_tokenizer,
            max_tokens: Some(max_tokens),
            merge_level: Some(merge_level),
            parallel: Some(parallel),
            patterns_len,
        }
    }

    pub fn new_hf2(
        patterns: Vec<Vec<String>>,
        max_tokens: Option<usize>,
        merge_level: Option<usize>,
        parallel: bool,
        model: Option<String>,
    ) -> Self {
        let patterns_len = patterns.len();
        let searchers: Vec<_> = patterns
            .clone()
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(model, Some(usize::MAX), false).unwrap(),
        };
        Self {
            searchers,
            tokenizer: hf_tokenizer,
            max_tokens,
            merge_level,
            parallel: Some(parallel),
            patterns_len,
        }
    }

    pub fn hf_splits(&self, data: &[u8]) -> Vec<SplitResultLite> {
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = crate::splitter::splitter_config::SplitterConfig::<HFTokenizer> {
            data,
            searchers: &self.searchers,
            max_len: self.max_tokens,
            merge_level: self.merge_level,
            tokenizer: Some(&self.tokenizer),
            patterns_len: self.patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, self.parallel, None, None);
        let tree = splitter.split();
        match term_tree(tree.clone(), data) {
            Ok(trm_tree) => {
                println!("{}", trm_tree);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        tree.get_results_lite(self.max_tokens, self.merge_level, data)
    }

    pub fn hf_splits_raw(&self, data: &[u8]) -> Vec<Split> {
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = crate::splitter::splitter_config::SplitterConfig::<HFTokenizer> {
            data,
            searchers: &self.searchers,
            max_len: self.max_tokens,
            merge_level: self.merge_level,
            tokenizer: Some(&self.tokenizer),
            patterns_len: self.patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, self.parallel, None, None);
        let tree = splitter.split();
        tree.get_node_splits(self.max_tokens, self.merge_level)
    }
}
