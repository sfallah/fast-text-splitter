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


#[derive(Debug, Clone)]
pub struct ConfigParams {
    pub pattern: Option<Vec<String>>,
    #[cfg(feature = "tokenizers")]
    pub model_path: Option<String>,
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
            .pattern(vec!["\n\n".to_string(), "\n".to_string()])
            .max_tokens(384)
            .max_depth(2)
            .merge_level(1)
            .parallel(true).build()
    }
    #[cfg(feature = "tokenizers")]
    pub fn hf_default() -> Self {
        Self::builder()
            .pattern(vec!["\n\n".to_string(), "\n".to_string()])
            .model_path("sentence-transformers/all-MiniLM-L6-v2".to_string())
            .max_tokens(512)
            .max_depth(2)
            .merge_level(1)
            .parallel(true).build()
    }
}

pub struct ConfigParamsBuilder {
    pattern: Option<Vec<String>>,
    #[cfg(feature = "tokenizers")]
    model_path: Option<String>,
    max_tokens: Option<usize>,
    max_depth: Option<usize>,
    merge_level: Option<usize>,
    parallel: Option<bool>,
}

impl ConfigParamsBuilder {
    pub fn new() -> Self {
        ConfigParamsBuilder {
            pattern: None,
            #[cfg(feature = "tokenizers")]
            model_path: None,
            max_tokens: None,
            max_depth: None,
            merge_level: None,
            parallel: None,
        }
    }

    pub fn pattern(mut self, pattern: Vec<String>) -> Self {
        self.pattern = Some(pattern);
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

    pub fn build(self) -> ConfigParams {
        ConfigParams {
            pattern: self.pattern,
            #[cfg(feature = "tokenizers")]
            model_path: self.model_path,
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
            aho_corasick: init_aho_corasick(config_params.pattern.clone()).unwrap().into(),
            tokenizer: WSTokenizer {},
            max_tokens: config_params.max_tokens.unwrap_or(384),
            max_depth: config_params.max_depth.unwrap_or(2),
            merge_level: config_params.merge_level,
            parallel: config_params.parallel.unwrap_or(true),
        }
    }
}

#[cfg(feature = "tokenizers")]
impl SplitterConfig<HFTokenizer> {
    pub fn from_params(config_params: ConfigParams) -> Self {
        Self {
            aho_corasick: init_aho_corasick(config_params.pattern.clone()).unwrap().into(),
            tokenizer: HFTokenizer {
                tokenizer: init_tokenizer(config_params.model_path).unwrap(),
            },
            max_tokens: config_params.max_tokens.unwrap_or(512),
            max_depth: config_params.max_depth.unwrap_or(2),
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

        assert_eq!(config_params.pattern.unwrap(), vec!["\n\n".to_string(), "\n".to_string()]);
        assert_eq!(config_params.max_tokens.unwrap(), 384);
        assert_eq!(config_params.max_depth.unwrap(), 2);
        assert_eq!(config_params.merge_level.unwrap(), 1);
        assert_eq!(config_params.parallel.unwrap(), true);
    }

    #[test]
    fn test_default_config_params() {
        let config_params = ConfigParams::ws_default();

        assert_eq!(config_params.pattern.unwrap(), vec!["\n\n".to_string(), "\n".to_string()]);
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
        assert_eq!(splitter_config.max_depth, 2);
        assert_eq!(splitter_config.merge_level, Some(1));
        assert_eq!(splitter_config.parallel, true);
    }

    #[test]
    #[cfg(feature = "tokenizers")]
    fn test_splitter_config_hf_tokenizer() {
        let config_params = ConfigParams::hf_default();
        let splitter_config = SplitterConfig::<HFTokenizer>::from_params(config_params);

        assert_eq!(splitter_config.max_tokens, 512);
        assert_eq!(splitter_config.max_depth, 2);
        assert_eq!(splitter_config.merge_level, Some(1));
        assert_eq!(splitter_config.parallel, true);
    }
}