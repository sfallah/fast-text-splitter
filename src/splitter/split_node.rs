use std::str::from_utf8;
use std::sync::Arc;

use aho_corasick::Span;

use crate::common::{chunk_encoding_splits, chunk_spans, span};
use crate::splitter::split_encoding::SplitEncoding;

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

    pub fn merge_splits(&self, merge_level: usize, max_len: Option<usize>) -> Vec<Span> {
        let max_len_check = max_len.is_some();
        let max_len_value = max_len.unwrap_or(0);

        let mut splits = Vec::new();
        if self.pattern_id >= merge_level {
            if max_len_check {
                splits.extend(chunk_spans(&self.all_leaves(), max_len_value));
            } else {
                splits.extend(self.all_leaves());
            }
        } else {
            for child in &self.children {
                splits.extend(child.merge_splits(merge_level, max_len));
            }
        }
        splits
    }

    pub fn merge_splits_encoding(
        &self,
        merge_level: usize,
        max_len: Option<usize>,
    ) -> Vec<(Span, Span)> {
        let max_len_check = max_len.is_some();
        let max_len_value = max_len.unwrap_or(0);

        let mut splits = Vec::new();
        if self.pattern_id >= merge_level {
            if max_len_check {
                let leaves = self.all_leaves_split();
                splits.extend(chunk_encoding_splits(&leaves, max_len_value));
            } else {
                splits.extend(self.all_leaves_split());
            }
        } else {
            for child in &self.children {
                splits.extend(child.merge_splits_encoding(merge_level, max_len));
            }
        }
        splits
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

    pub fn all_leaves_split(&self) -> Vec<(Span, Span)> {
        if self.children.is_empty() {
            vec![(
                self.split_tokens_span.clone().unwrap_or(span(0,0)),
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
