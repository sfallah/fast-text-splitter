use pyo3::prelude::*;

#[cfg(feature = "tokenizers")]
use crate::common::TokensResults;
use crate::{config, text_split_parallel};
use crate::config::ConfigParams;
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

#[pyfunction]
pub fn text_split_ws(data: &str, py_conf_params: &PyConfigParams) -> Vec<PySplitResults> {
    println!("ConfigParams= {:?}", py_conf_params);
    let conf_params: ConfigParams = py_conf_params.clone().into();
    let conf = config::SplitterConfig::<WSTokenizer>::from_params(&conf_params);

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
        }
    }
}


