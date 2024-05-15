use crate::ac_matches::init_aho_corasick;
use crate::encodings::Tokenize;
#[cfg(feature = "tokenizers")]
use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
use crate::ws_tokenizer::WSTokenizer;
use aho_corasick::AhoCorasick;
use pyo3::{pyclass, pymethods};

#[derive(Debug, Clone)]
pub struct SplitterConfig<T: Tokenize + Sync> {
    pub aho_corasick: AhoCorasick,
    pub tokenizer: T,
    pub max_tokens: usize,
    pub max_depth: usize,
    pub merge_level: Option<usize>,
    pub parallel: bool,
}

impl Default for SplitterConfig<WSTokenizer> {
    fn default() -> Self {
        SplitterConfig {
            aho_corasick: init_aho_corasick(None).into(),
            tokenizer: WSTokenizer {},
            max_tokens: 384,
            max_depth: 2,
            merge_level: Some(1),
            parallel: false,
        }
    }
}

#[cfg(feature = "tokenizers")]
impl Default for SplitterConfig<HFTokenizer> {
    fn default() -> Self {
        SplitterConfig {
            aho_corasick: init_aho_corasick(None).into(),
            tokenizer: HFTokenizer {
                tokenizer: init_tokenizer(None).unwrap(),
            },
            max_tokens: 512,
            max_depth: 2,
            merge_level: Some(1),
            parallel: true,
        }
    }
}

impl SplitterConfig<WSTokenizer> {
    pub fn new_ws_config(
        pattern: Option<&[&str]>,
        max_tokens: usize,
        max_depth: usize,
        merge_level: Option<usize>,
        parallel: bool,
    ) -> Self {
        SplitterConfig {
            aho_corasick: init_aho_corasick(pattern).into(),
            tokenizer: WSTokenizer {},
            max_tokens,
            max_depth,
            merge_level,
            parallel,
        }
    }
}

#[cfg(feature = "tokenizers")]
impl SplitterConfig<HFTokenizer> {
    pub fn new_hf_config(
        pattern: Option<&[&str]>,
        model_path: Option<String>,
        max_tokens: usize,
        max_depth: usize,
        merge_level: Option<usize>,
        parallel: bool,
    ) -> Self {
        SplitterConfig {
            aho_corasick: init_aho_corasick(pattern).into(),
            tokenizer: HFTokenizer {
                tokenizer: init_tokenizer(model_path).unwrap(),
            },
            max_tokens,
            max_depth,
            merge_level,
            parallel,
        }
    }
}

#[pyclass]
pub struct  PySplitterConfig {
    pub pattern: Option<Vec<String>>,
    pub model_path: Option<String>,
    pub max_tokens: Option<usize>,
    pub max_depth: Option<usize>,
    pub merge_level: Option<usize>,
    pub parallel: Option<bool>,
}

#[pyclass]
pub struct PySplitterConfigBuilder {
    pattern: Option<Vec<String>>,
    model_path: Option<String>,
    max_tokens: Option<usize>,
    max_depth: Option<usize>,
    merge_level: Option<usize>,
    parallel: Option<bool>,
}

#[pymethods]
impl PySplitterConfigBuilder {
    pub fn new() -> PySplitterConfigBuilder {
        PySplitterConfigBuilder {
            pattern: None,
            model_path: None,
            max_tokens: None,
            max_depth: None,
            merge_level: None,
            parallel: None,
        }
    }

    pub fn pattern(mut self, pattern: Vec<String>) -> PySplitterConfigBuilder {
        self.pattern = Some(pattern);
        self
    }

    pub fn model_path(mut self, model_path: String) -> PySplitterConfigBuilder {
        self.model_path = Some(model_path);
        self
    }

    pub fn max_tokens(mut self, max_tokens: usize) -> PySplitterConfigBuilder {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn max_depth(mut self, max_depth: usize) -> PySplitterConfigBuilder {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn merge_level(mut self, merge_level: usize) -> PySplitterConfigBuilder {
        self.merge_level = Some(merge_level);
        self
    }

    pub fn parallel(mut self, parallel: bool) -> PySplitterConfigBuilder {
        self.parallel = Some(parallel);
        self
    }

    pub fn build(self) -> PySplitterConfig {
        PySplitterConfig {
            pattern: self.pattern,
            model_path: self.model_path,
            max_tokens: self.max_tokens,
            max_depth: self.max_depth,
            merge_level: self.merge_level,
            parallel: self.parallel,
        }
    }
}

