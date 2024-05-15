use crate::ac_matches::init_aho_corasick;
use crate::encodings::Tokenize;
#[cfg(feature = "tokenizers")]
use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
use crate::ws_tokenizer::WSTokenizer;
use aho_corasick::AhoCorasick;

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
