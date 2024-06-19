use aho_corasick::Span;
use crate::encodings::Tokenize;
#[cfg(feature = "tokenizers")]
use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
use crate::pattern_search::pattern_searcher::PatternSearcher;
use crate::ws_tokenizer::WSTokenizer;
use crate::splitter::split_node::utils::SplitResultLite;
use crate::splitter::Splitter;

pub struct SplitterLiteConfig<'a, T: Tokenize + Sync> {
    pub patterns: Vec<Vec<&'a str>>,
    pub searchers: Vec<PatternSearcher<'a>>,
    pub tokenizer: T,
    pub max_tokens: Option<usize>,
    pub merge_level: Option<usize>,
    pub parallel: Option<bool>,
}


impl<'a> SplitterLiteConfig<'a, WSTokenizer> {
    pub fn new_ws(patterns: &'a Vec<Vec<&str>>, max_tokens: usize, merge_level: usize, parallel: bool) -> Self {
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
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
        let config =
            crate::splitter::splitter_config::SplitterConfig::<WSTokenizer> {
                data,
                patterns: &self.patterns,
                searchers: &self.searchers,
                max_len: self.max_tokens,
                tokenizer: Some(&self.tokenizer),
            };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();
        tree.get_results_lite(self.max_tokens, data)
    }
}

impl<'a> SplitterLiteConfig<'a, HFTokenizer> {
    pub fn new_hf(patterns: &'a Vec<Vec<&str>>, max_tokens: usize, merge_level: usize, parallel: bool, model: Option<String>) -> Self {
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
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
        let config =
            crate::splitter::splitter_config::SplitterConfig::<HFTokenizer> {
                data,
                patterns: &self.patterns,
                searchers: &self.searchers,
                max_len: self.max_tokens,
                tokenizer: Some(&self.tokenizer),
            };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();
        tree.get_results_lite(self.max_tokens, data)
    }
}

#[derive(Debug, Clone)]
pub struct SplitterConfig<T: Tokenize + Sync> {
    pub pattern: Vec<Vec<String>>,
    pub tokenizer: T,
    pub max_tokens: usize,
    pub merge_level: Option<usize>,
    pub parallel: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigParams {
    pub patterns: Option<Vec<Vec<String>>>,
    #[cfg(feature = "tokenizers")]
    pub model_path: Option<String>,
    #[cfg(feature = "tokenizers")]
    pub tokenizer_max_len: Option<usize>,
    pub max_tokens: Option<usize>,
    pub max_depth: Option<usize>,
    pub merge_level: Option<usize>,
    pub parallel: Option<bool>,
    pub conf_type: Option<String>,
}

impl ConfigParams {
    pub fn builder() -> ConfigParamsBuilder {
        ConfigParamsBuilder::new()
    }
}

impl ConfigParams {
    pub fn ws_default() -> Self {
        Self::builder()
            .pattern(vec![vec!["\n\n".to_string()], vec!["\n".to_string()]])
            .max_tokens(384)
            .max_depth(2)
            .merge_level(1)
            .parallel(true)
            .build()
    }
    #[cfg(feature = "tokenizers")]
    pub fn hf_default() -> Self {
        Self::builder()
            .pattern(vec![vec!["\n\n".to_string()], vec!["\n".to_string()]])
            .model_path("sentence-transformers/all-MiniLM-L6-v2".to_string())
            .max_tokens(512)
            .max_depth(2)
            .merge_level(1)
            .parallel(true)
            .build()
    }
}

pub struct ConfigParamsBuilder {
    patterns: Option<Vec<Vec<String>>>,
    #[cfg(feature = "tokenizers")]
    model_path: Option<String>,
    #[cfg(feature = "tokenizers")]
    tokenizer_max_len: Option<usize>,
    max_tokens: Option<usize>,
    max_depth: Option<usize>,
    merge_level: Option<usize>,
    parallel: Option<bool>,
}

impl ConfigParamsBuilder {
    pub fn new() -> Self {
        ConfigParamsBuilder {
            patterns: None,
            #[cfg(feature = "tokenizers")]
            model_path: None,
            #[cfg(feature = "tokenizers")]
            tokenizer_max_len: None,
            max_tokens: None,
            max_depth: None,
            merge_level: None,
            parallel: None,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Vec<String>>) -> Self {
        self.patterns = Some(pattern);
        self
    }

    #[cfg(feature = "tokenizers")]
    pub fn model_path(mut self, model_path: String) -> Self {
        self.model_path = Some(model_path);
        self
    }

    pub fn max_tokens(mut self, max_tokens: usize) -> ConfigParamsBuilder {
        self.max_tokens = Some(max_tokens);
        self
    }
    pub fn max_depth(mut self, max_depth: usize) -> ConfigParamsBuilder {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn merge_level(mut self, merge_level: usize) -> ConfigParamsBuilder {
        self.merge_level = Some(merge_level);
        self
    }

    pub fn parallel(mut self, parallel: bool) -> ConfigParamsBuilder {
        self.parallel = Some(parallel);
        self
    }

    #[cfg(feature = "tokenizers")]
    pub fn tokenizer_max_len(mut self, tokenizer_max_len: usize) -> ConfigParamsBuilder {
        self.tokenizer_max_len = Some(tokenizer_max_len);
        self
    }

    pub fn build(self) -> ConfigParams {
        ConfigParams {
            patterns: self.patterns,
            #[cfg(feature = "tokenizers")]
            model_path: self.model_path,
            #[cfg(feature = "tokenizers")]
            tokenizer_max_len: self.tokenizer_max_len,
            max_tokens: self.max_tokens,
            max_depth: self.max_depth,
            merge_level: self.merge_level,
            parallel: self.parallel,
            conf_type: None, // None for now
        }
    }
}

impl SplitterConfig<WSTokenizer> {
    pub fn from_params(config_params: &ConfigParams) -> Self {
        Self {
            pattern: config_params
                .patterns
                .clone()
                .unwrap_or(vec![vec!["\n\n".to_string()], vec!["\n".to_string()]]),
            tokenizer: WSTokenizer {},
            max_tokens: config_params.max_tokens.unwrap_or(384),
            merge_level: config_params.merge_level,
            parallel: config_params.parallel.unwrap_or(true),
        }
    }
}

#[cfg(feature = "tokenizers")]
impl SplitterConfig<HFTokenizer> {
    pub fn from_params(config_params: ConfigParams) -> Self {
        Self {
            pattern: config_params
                .patterns
                .unwrap_or(vec![vec!["\n\n".to_string()], vec!["\n".to_string()]]),
            tokenizer: HFTokenizer {
                tokenizer: init_tokenizer(
                    config_params.model_path,
                    config_params.tokenizer_max_len,
                    false,
                )
                .unwrap(),
            },
            max_tokens: config_params.max_tokens.unwrap_or(512),
            merge_level: config_params.merge_level,
            parallel: config_params.parallel.unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_params_builder() {
        let config_params = ConfigParams::ws_default();

        assert_eq!(
            config_params.patterns.unwrap(),
            vec![vec!["\n\n".to_string()], vec!["\n".to_string()]],
        );
        assert_eq!(config_params.max_tokens.unwrap(), 384);
        assert_eq!(config_params.max_depth.unwrap(), 2);
        assert_eq!(config_params.merge_level.unwrap(), 1);
        assert_eq!(config_params.parallel.unwrap(), true);
    }

    #[test]
    fn test_default_config_params() {
        let config_params = ConfigParams::ws_default();

        assert_eq!(
            config_params.patterns.unwrap(),
            vec![vec!["\n\n".to_string()], vec!["\n".to_string()]]
        );
        assert_eq!(config_params.max_tokens.unwrap(), 384);
        assert_eq!(config_params.max_depth.unwrap(), 2);
        assert_eq!(config_params.merge_level.unwrap(), 1);
        assert_eq!(config_params.parallel.unwrap(), true);
    }

    #[test]
    fn test_splitter_config_ws_tokenizer() {
        let config_params = ConfigParams::ws_default();
        let splitter_config = SplitterConfig::<WSTokenizer>::from_params(&config_params);

        assert_eq!(splitter_config.max_tokens, 384);
        assert_eq!(splitter_config.merge_level, Some(1));
        assert_eq!(splitter_config.parallel, true);
    }

    #[test]
    #[cfg(feature = "tokenizers")]
    fn test_splitter_config_hf_tokenizer() {
        let config_params = ConfigParams::hf_default();
        let splitter_config = SplitterConfig::<HFTokenizer>::from_params(config_params);

        assert_eq!(splitter_config.max_tokens, 512);
        assert_eq!(splitter_config.merge_level, Some(1));
        assert_eq!(splitter_config.parallel, true);
    }
}
