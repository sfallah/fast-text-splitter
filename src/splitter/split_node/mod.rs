pub mod utils;

use std::str::from_utf8;
use std::sync::Arc;

use aho_corasick::Span;

use crate::common::{
    chunk_encoding_splits, merge_split_results, Split, SplitResults,
    TokensResultLite,
};
use crate::splitter::split_encoding::SplitEncoding;
use crate::splitter::split_node::utils::{merge_split_result_lite, SplitResultLite};

#[derive(Clone)]
pub struct SplitNode {
    pub split_data_span: Span,
    pub pattern_id: usize,
    pub children: Vec<SplitNode>,
    pub split_encoding: Option<Arc<SplitEncoding>>,
    pub split_tokens_span: Option<Span>,
}

impl SplitNode {
    pub fn new(
        pattern_id: usize,
        split_data_span: Span,
        children: Vec<SplitNode>,
        split_encoding: Option<Arc<SplitEncoding>>,
        split_tokens_span: Option<Span>,
    ) -> Self {
        SplitNode {
            split_data_span,
            pattern_id,
            children,
            split_encoding,
            split_tokens_span,
        }
    }
    pub fn to_string_with_indent(&self, data: &[u8], indent: usize, reconstruct: bool) -> String {
        let indent_str = " ".repeat(indent);

        if !self.children.is_empty() {
            let mut result = format!(
                "{}Node:\n{} pattern_id: {} \n{} len: {} \n{} children:\n",
                indent_str,
                indent_str,
                self.pattern_id,
                indent_str,
                self.split_data_span.len(),
                indent_str
            );
            self.children.iter().for_each(|child| {
                result.push_str(&child.to_string_with_indent(data, indent + 4, reconstruct));
            });
            if reconstruct {
                result.push_str(&format!(
                    "{} orig: {:?}\n",
                    indent_str,
                    self.reconstruct(data)
                ));
            }
            result
        } else {
            let mut result = format!(
                "{}Leaf: \n{} pattern_id: {} \n{} len: {} \n{} tokens_span: {:?}\n",
                indent_str,
                indent_str,
                self.pattern_id,
                indent_str,
                self.split_data_span.len(),
                indent_str,
                self.split_tokens_span
            );
            if reconstruct {
                result.push_str(&format!(
                    "{} orig: {:?}\n",
                    indent_str,
                    self.reconstruct(data)
                ));
            }
            result
        }
    }
    pub fn to_string(&self, data: &[u8], reconstruct: bool) -> String {
        self.to_string_with_indent(data, 0, reconstruct)
    }

    pub fn reconstruct(&self, data: &[u8]) -> String {
        from_utf8(&data[self.split_data_span.start..self.split_data_span.end])
            .unwrap()
            .to_string()
    }

    pub fn get_results(&self, max_len: Option<usize>, data: &str) -> Vec<SplitResults> {
        let merge_level = self.leaf_level().unwrap();
        let split_res = self.merge_encoding_result(merge_level, max_len, data);
        merge_split_results(&split_res, max_len.unwrap())
    }

    pub fn get_results_lite(&self, max_len: Option<usize>, data: &[u8]) -> Vec<SplitResultLite> {
        let merge_level = self.leaf_level().unwrap();
        let split_res = self.merge_enc_lite_result(merge_level, max_len);
        if let Some(max_len) = max_len {
            merge_split_result_lite(&split_res, max_len, data)
        } else {
            split_res
                .iter()
                .map(|res| SplitResultLite {
                    tokens: res.ids.map_or_else(Vec::new, |ids| ids.to_vec()),
                    split_string: from_utf8(&data[res.data_span.range()]).unwrap().to_string(),
                })
                .collect()
        }
    }

    pub fn merge_encoding_result(
        &self,
        merge_level: usize,
        max_len: Option<usize>,
        data: &str,
    ) -> Vec<SplitResults> {
        let max_len_check = max_len.is_some();
        let max_len_value = max_len.unwrap_or(0);

        let mut split_results = Vec::new();
        if self.pattern_id >= merge_level {
            let raw_splits = if max_len_check {
                let leaves = self.all_leaves_split();
                chunk_encoding_splits(&leaves, max_len_value)
            } else {
                self.all_leaves_split()
            };

            let splits: Vec<_> = raw_splits
                .iter()
                .map(|(tokens_span, data_span)| Split {
                    tokens_span: tokens_span.clone().unwrap(),
                    data_span: Some(data_span.clone()),
                })
                .collect();
            let split_res = self
                .split_encoding
                .clone()
                .unwrap()
                .encoding
                .to_split_results(&splits, data);

            split_results.extend(split_res);
        } else {
            for child in &self.children {
                split_results.extend(child.merge_encoding_result(merge_level, max_len, data));
            }
        }
        split_results
    }

    pub fn merge_enc_lite_result(
        &self,
        merge_level: usize,
        max_len: Option<usize>,
    ) -> Vec<TokensResultLite> {
        let max_len_check = max_len.is_some();
        let max_len_value = max_len.unwrap_or(0);

        let mut split_results = Vec::new();
        if self.pattern_id >= merge_level {
            let raw_splits = if max_len_check {
                let leaves = self.all_leaves_split();
                chunk_encoding_splits(&leaves, max_len_value)
            } else {
                self.all_leaves_split()
            };

            let splits: Vec<_> = raw_splits
                .iter()
                .map(|(tokens_span, data_span)| Split {
                    tokens_span: tokens_span.clone().unwrap(),
                    data_span: Some(data_span.clone()),
                })
                .collect();

            if let Some(encoding) = &self.split_encoding {
                let split_res = encoding.encoding.to_lite_results(&splits);
                split_results.extend(split_res);
            } else {
                //FIXME: This is a temporary fix, we need to adapt it to work with like for the HFEncoding nad WSEncoding
                let split_res = splits.iter().map(|split| TokensResultLite {
                    data_span: split.data_span.clone().unwrap(),
                    ids: None,
                    offsets: None,
                });
                split_results.extend(split_res);
            }

            /*
            let split_res = self
                .split_encoding
                .as_ref()
                .unwrap()
                .encoding
                .to_lite_results(&splits);

            split_results.extend(split_res);

             */
        } else {
            for child in &self.children {
                split_results.extend(child.merge_enc_lite_result(merge_level, max_len));
            }
        }
        split_results
    }

    //FIXME: This needs more testing, and see if it can be combined with merge_splits
    pub fn leaf_level(&self) -> Option<usize> {
        if self.children.is_empty() {
            Some(self.pattern_id)
        } else {
            let leaf_found = self.children.iter().find(|child| child.children.is_empty());
            if let Some(leaf) = leaf_found {
                Some(leaf.pattern_id)
            } else {
                self.children
                    .iter()
                    .find(|child| child.leaf_level().is_some())
                    .map(|leaf| leaf.pattern_id)
            }
        }
    }

    pub fn all_leaves(&self) -> Vec<Span> {
        if self.children.is_empty() {
            vec![self.split_data_span]
        } else {
            self.children
                .iter()
                .flat_map(|child| child.all_leaves())
                .collect()
        }
    }

    pub fn all_leaves_split(&self) -> Vec<(Option<Span>, Span)> {
        if self.children.is_empty() {
            vec![(
                self.split_tokens_span.clone(),
                self.split_data_span.clone(),
            )]
        } else {
            self.children
                .iter()
                .flat_map(|child| child.all_leaves_split())
                .collect()
        }
    }
}
