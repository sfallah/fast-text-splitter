use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use lazy_static::lazy_static;

#[cfg(feature = "tokenizers")]
use crate::common::TokensResults;
use crate::{config, text_split_parallel};
use crate::config::{ConfigParams, SplitterConfig};
use crate::hf_tokenizer::HFTokenizer;
use crate::ws_tokenizer::WSTokenizer;

#[pyclass]
#[derive(Default, PartialEq, Debug, Clone)]
#[cfg(feature = "tokenizers")]
pub struct PyTokensResults {
    #[pyo3(get)]
    pub ids: Vec<u32>,
    #[pyo3(get)]
    pub type_ids: Vec<u32>,
    #[pyo3(get)]
    pub attention_mask: Vec<u32>,
    #[pyo3(get)]
    pub offsets: Vec<(usize, usize)>,
}

#[pyclass]
pub struct PySplitResults {
    #[pyo3(get)]
    pub results: Option<PyTokensResults>,
    #[pyo3(get)]
    pub split_strings: String,
}


lazy_static! {
    static ref CACHE: Arc<Mutex<HashMap<u64, Arc<SplitterConfig<WSTokenizer>>>>> = Arc::new(Mutex::new(HashMap::new()));
}


fn splitter_config_cache(cache_params: &PyConfigParams) -> Arc<SplitterConfig<WSTokenizer>> {
    let mut hasher = DefaultHasher::new();
    cache_params.hash(&mut hasher);
    let cache_key = hasher.finish();

    // Acquire the lock and check if the result is already in the cache
    {
        let cache = CACHE.lock().unwrap();
        if let Some(cached_conf) = cache.get(&cache_key) {
            return cached_conf.clone();
        }
    }

    // If the result is not in the cache, compute it
    let conf_params: ConfigParams = cache_params.clone().into();
    let conf = config::SplitterConfig::<WSTokenizer>::from_params(&conf_params);
    let conf = Arc::new(conf);

    // Store the result in the cache
    {
        let mut cache = CACHE.lock().unwrap();
        cache.insert(cache_key, conf.clone());
    }

    conf

}

#[pyfunction]
pub fn text_split_ws(data: &str, py_conf_params: &PyConfigParams) -> Vec<PySplitResults> {
    println!("ConfigParams= {:?}", py_conf_params);
    
    // Basic Caching
    let mut cache_params = py_conf_params.clone();
    cache_params.conf_type = Some("WS".to_string());

    let conf = splitter_config_cache(&cache_params);

    text_split_parallel(&conf, data).iter()
        .map(|x| PySplitResults {
            results: None,
            split_strings: x.split_strings.clone(),
        }
        ).collect()
}

#[cfg(feature = "tokenizers")]
#[pyfunction]
pub fn text_split_hf(data: &str, py_conf_params: &PyConfigParams) -> Vec<PySplitResults> {
    println!("ConfigParams= {:?}", py_conf_params);

    let conf_params: ConfigParams = py_conf_params.clone().into();
    let conf = config::SplitterConfig::<HFTokenizer>::from_params(conf_params);

    text_split_parallel(&conf, data).iter()
        .map(|x| PySplitResults {
            results: x.results.clone().map(|t| t.into()),
            split_strings: x.split_strings.clone(),
        }
        ).collect()
}


#[pyfunction]
#[pyo3(
    signature = (merge_level = None, patterns = vec ! [], max_tokens = 512, max_depth = 2, parallel = true)
)]
pub fn py_ws_config_params(
    merge_level: Option<usize>,
    patterns: Vec<String>,
    max_tokens: usize,
    max_depth: usize,
    parallel: bool,
) -> PyConfigParams {
    ConfigParams::builder()
        .pattern(patterns)
        .merge_level(merge_level.unwrap_or(0))
        .max_tokens(max_tokens)
        .max_depth(max_depth)
        .parallel(parallel)
        .build().into()
}

#[cfg(feature = "tokenizers")]
#[pyfunction]
#[pyo3(
    signature = (model_path = None, merge_level = None, patterns = vec ! [], max_tokens = 512, max_depth = 2, parallel = true)
)]
pub fn py_hf_config_params(
    model_path: Option<String>,
    merge_level: Option<usize>,
    patterns: Vec<String>,
    max_tokens: usize,
    max_depth: usize,
    parallel: bool,
) -> PyConfigParams {
    ConfigParams::builder()
        .model_path(model_path.unwrap_or("".to_string()))
        .pattern(patterns)
        .merge_level(merge_level.unwrap_or(0))
        .max_tokens(max_tokens)
        .max_depth(max_depth)
        .parallel(parallel)
        .build().into()
}

#[pymodule]
fn fast_text_splitter(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(text_split_ws, m)?)?;
    m.add_function(wrap_pyfunction!(py_ws_config_params, m)?)?;
    m.add_class::<PyConfigParams>()?;
    m.add_class::<PySplitResults>()?;
    #[cfg(feature = "tokenizers")]
    m.add_class::<PyTokensResults>()?;
    #[cfg(feature = "tokenizers")]
    m.add_function(wrap_pyfunction!(py_hf_config_params, m)?)?;
    #[cfg(feature = "tokenizers")]
    m.add_function(wrap_pyfunction!(text_split_hf, m)?)?;
    Ok(())
}

impl From<TokensResults> for PyTokensResults {
    fn from(py_tokens: TokensResults) -> Self {
        PyTokensResults {
            ids: py_tokens.ids,
            type_ids: py_tokens.type_ids,
            attention_mask: py_tokens.attention_mask,
            offsets: py_tokens.offsets,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct PyConfigParams {
    #[pyo3(get)]
    pub pattern: Option<Vec<String>>,
    #[cfg(feature = "tokenizers")]
    #[pyo3(get)]
    pub model_path: Option<String>,
    #[pyo3(get)]
    pub max_tokens: Option<usize>,
    #[pyo3(get)]
    pub max_depth: Option<usize>,
    #[pyo3(get)]
    pub merge_level: Option<usize>,
    #[pyo3(get)]
    pub parallel: Option<bool>,
    #[pyo3(get)]
    pub conf_type: Option<String>,
}

impl Hash for PyConfigParams {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hasher = DefaultHasher::new();
        self.pattern.hash(&mut hasher);
        #[cfg(feature = "tokenizers")]
        self.model_path.hash(&mut hasher);
        self.max_tokens.hash(&mut hasher);
        self.max_depth.hash(&mut hasher);
        self.merge_level.hash(&mut hasher);
        self.parallel.hash(&mut hasher);
        hasher.finish().hash(state);
    }
}

impl From<ConfigParams> for PyConfigParams {
    fn from(conf_params: ConfigParams) -> Self {
        PyConfigParams {
            pattern: conf_params.pattern,
            #[cfg(feature = "tokenizers")]
            model_path: conf_params.model_path,
            max_tokens: conf_params.max_tokens,
            max_depth: conf_params.max_depth,
            merge_level: conf_params.merge_level,
            parallel: conf_params.parallel,
            conf_type: None, // None for now
        }
    }
}

impl From<PyConfigParams> for ConfigParams {
    fn from(py_conf_params: PyConfigParams) -> Self {
        ConfigParams {
            pattern: py_conf_params.pattern,
            #[cfg(feature = "tokenizers")]
            model_path: py_conf_params.model_path,
            max_tokens: py_conf_params.max_tokens,
            max_depth: py_conf_params.max_depth,
            merge_level: py_conf_params.merge_level,
            parallel: py_conf_params.parallel,
            conf_type: None, // None for now
        }
    }
}


