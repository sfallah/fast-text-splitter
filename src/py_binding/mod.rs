use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::config::SplitterLiteConfig;
use crate::encodings::NoneTokenizer;
use crate::hf_tokenizer::HFTokenizer;
use crate::ws_tokenizer::WSTokenizer;

#[pyclass]
struct PySplitLiteResult {
    #[pyo3(get)]
    tokens: Vec<u32>,
    #[pyo3(get)]
    text: String,
}

#[pyclass]
struct PyNoneSplitterConfig {
    inner: SplitterLiteConfig<NoneTokenizer>,
}

#[pymethods]
impl PyNoneSplitterConfig {
    #[new]
    fn new(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
    ) -> Self {
        PyNoneSplitterConfig {
            inner: SplitterLiteConfig::new_none(patterns, max_tokens, merge_level, parallel),
        }
    }

    fn splits(&self, data: String) -> PyResult<Vec<PySplitLiteResult>> {
        Ok(self
            .inner
            .len_splits(data.as_bytes())
            .iter()
            .map(|x| PySplitLiteResult {
                tokens: x.tokens.clone(),
                text: x.split_string.clone(),
            })
            .collect())
    }
}

#[pyclass]
struct PyWSSplitterConfig {
    inner: SplitterLiteConfig<WSTokenizer>,
}

#[pymethods]
impl PyWSSplitterConfig {
    #[new]
    fn new(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
        ascii: bool,
    ) -> Self {
        PyWSSplitterConfig {
            inner: SplitterLiteConfig::new_ws(patterns, max_tokens, merge_level, parallel, ascii),
        }
    }

    fn splits(&self, data: String) -> PyResult<Vec<PySplitLiteResult>> {
        Ok(self
            .inner
            .ws_splits(data.as_bytes())
            .iter()
            .map(|x| PySplitLiteResult {
                tokens: x.tokens.clone(),
                text: x.split_string.clone(),
            })
            .collect())
    }
}

#[pyclass]
struct PyHFSplitterConfig {
    inner: SplitterLiteConfig<HFTokenizer>,
}

#[pymethods]
impl PyHFSplitterConfig {
    #[new]
    fn new(
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        merge_level: usize,
        parallel: bool,
    ) -> Self {
        PyHFSplitterConfig {
            inner: SplitterLiteConfig::new_hf(patterns, max_tokens, merge_level, parallel, None),
        }
    }

    fn splits(&self, data: String) -> PyResult<Vec<PySplitLiteResult>> {
        println!("ws_splits");
        println!("data:{:?}", &data);
        Ok(self
            .inner
            .hf_splits(data.as_bytes())
            .iter()
            .map(|x| PySplitLiteResult {
                tokens: x.tokens.clone(),
                text: x.split_string.clone(),
            })
            .collect())
    }
}

#[pyfunction]
fn create_none_splitter(
    patterns: Vec<Vec<String>>,
    max_tokens: usize,
    merge_level: usize,
    parallel: bool,
) -> PyNoneSplitterConfig {
    PyNoneSplitterConfig::new(patterns, max_tokens, merge_level, parallel)
}

#[pyfunction]
fn create_ws_splitter(
    patterns: Vec<Vec<String>>,
    max_tokens: usize,
    merge_level: usize,
    parallel: bool,
    ascii: bool,
) -> PyWSSplitterConfig {
    PyWSSplitterConfig::new(patterns, max_tokens, merge_level, parallel, ascii)
}

#[pyfunction]
fn create_hf_splitter(
    patterns: Vec<Vec<String>>,
    max_tokens: usize,
    merge_level: usize,
    parallel: bool,
) -> PyHFSplitterConfig {
    PyHFSplitterConfig::new(patterns, max_tokens, merge_level, parallel)
}

#[pymodule]
fn fast_text_splitter(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNoneSplitterConfig>()?;
    m.add_class::<PyWSSplitterConfig>()?;
    m.add_class::<PyHFSplitterConfig>()?;
    m.add_class::<PySplitLiteResult>()?;
    m.add_function(wrap_pyfunction!(create_none_splitter, m)?)?;
    m.add_function(wrap_pyfunction!(create_ws_splitter, m)?)?;
    m.add_function(wrap_pyfunction!(create_hf_splitter, m)?)?;

    Ok(())
}
