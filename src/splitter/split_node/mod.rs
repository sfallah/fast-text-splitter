pub mod utils;
pub mod visualization;

use std::str::from_utf8;
use std::sync::Arc;

use aho_corasick::Span;

use crate::common::split::Split;
use crate::splitter::split_encoding::SplitEncoding;
use crate::splitter::split_node::utils::{add_splits, add_splits_no_merge, attach_pattern_nodes, merge_splits, SplitResultLite};

#[derive(Clone)]
pub struct SplitNode {
    pub split_data_span: Span,
    pub pattern_id: usize,
    pub children: Vec<SplitNode>,
    pub split_encoding: Option<Arc<SplitEncoding>>,
    pub split_tokens_span: Option<Span>,
    pattern_node: bool,
}

impl AsRef<SplitNode> for SplitNode {
    fn as_ref(&self) -> &SplitNode {
        self
    }
}

impl std::fmt::Debug for SplitNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.children.is_empty() {
            f.debug_struct("Node")
                .field("pattern_id", &format_args!("{:#?}", &self.pattern_id))
                .field("data_span", &format_args!("{:#?}", &self.split_data_span))
                .field("len", &format_args!("{:#?}", &self.split_data_span.len()))
                //.field("tokens_span",&format_args!("{:#?}", &self.split_tokens_span))
                .field(
                    "no_tokens",
                    &format_args!(
                        "{:#?}",
                        &self.split_tokens_span.map_or_else(|| 0, |span| span.len())
                    ),
                )
                .field("no_children", &format_args!("{:#?}", &self.children.len()))
                .finish()
        } else {
            f.debug_struct("Leaf")
                .field("pattern_id", &format_args!("{:#?}", &self.pattern_id))
                .field("data_span", &format_args!("{:#?}", &self.split_data_span))
                .field("len", &format_args!("{:#?}", &self.split_data_span.len()))
                //.field("tokens_span", &format_args!("{:#?}", &self.split_tokens_span))
                .field(
                    "no_tokens",
                    &format_args!(
                        "{:#?}",
                        &self.split_tokens_span.map_or_else(|| 0, |span| span.len())
                    ),
                )
                .finish()
        }
    }
}

impl SplitNode {
    pub fn new(
        pattern_id: usize,
        split_data_span: Span,
        children: Vec<SplitNode>,
        split_encoding: Option<Arc<SplitEncoding>>,
        split_tokens_span: Option<Span>,
        pattern_node: bool,
    ) -> Self {
        SplitNode {
            split_data_span,
            pattern_id,
            children,
            split_encoding,
            split_tokens_span,
            pattern_node,
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

    pub fn get_results_lite(
        &self,
        max_len: Option<usize>,
        merge_level: Option<usize>,
        data: &[u8],
    ) -> Vec<SplitResultLite> {
        let splits = self.get_node_splits(max_len, merge_level);
        splits
            .iter()
            .map(|split| {
                let mut tokens = Vec::new();
                if let Some(tokens_results) = split.tokens_results.as_ref() {
                    tokens_results.iter().for_each(|tokens_result| {
                        if tokens_result.ids.is_some() {
                            tokens.extend_from_slice(tokens_result.ids.as_ref().unwrap());
                        }
                    });
                }
                SplitResultLite {
                    tokens,
                    split_string: from_utf8(&data[split.data_span.unwrap().range()])
                        .unwrap()
                        .to_string(),
                }
            })
            .collect()
    }

    pub fn get_node_splits(
        &self,
        max_tokens: Option<usize>,
        merge_level: Option<usize>,
    ) -> Vec<Split> {
        if self.children.is_empty() {
            let tokens_no = self.split_tokens_span.map(|span| span.len());
            let tokens_results = self.split_encoding.as_ref().map(|encoding| {
                vec![Arc::new(
                    encoding.encoding.to_lite_results(self.split_tokens_span),
                )]
            });
            vec![Split {
                pattern_id: self.pattern_id,
                tokens_no,
                data_span: Some(self.split_data_span.clone()),
                pattern_node: self.pattern_node,
                tokens_results,
                tokenized: self.split_encoding.is_some(),
            }]
        } else {
            if let Some(max_len) = max_tokens {
                // add current pattern_id to all splits
                //??? if self.pattern_id <= merge_level
                let children_all_leaves: bool =
                    self.children.iter().all(|child| child.children.is_empty());
                if children_all_leaves {
                    let children_splits: Vec<Split> = self
                        .children
                        .iter()
                        .map(|child| child.get_node_splits(max_tokens, merge_level))
                        .flatten()
                        .collect();
                    // if self.pattern_id > merge_level
                    // attach pattern_nodes only
                    if let Some(merge_level) = merge_level {
                        if self.pattern_id <= merge_level {
                            return attach_pattern_nodes(&children_splits, max_len);
                        }
                    }
                    merge_splits(&children_splits, max_len)
                } else {
                    // add current pattern_id to all splits
                    //??? if self.pattern_id <= merge_level
                    let children_splits: Vec<Vec<Split>> = self
                        .children
                        .iter()
                        .map(|child| child.get_node_splits(max_tokens, merge_level))
                        .collect();
                    // if self.pattern_id > merge_level don't merge
                    // attach pattern_nodes to the last split
                    children_splits
                        .into_iter()
                        .fold(Vec::new(), |mut acc, child_splits| {
                            if let Some(merge_level) = merge_level {
                                if self.pattern_id < merge_level {
                                    add_splits_no_merge(&mut acc, &child_splits.clone(), max_len);
                                    return acc;
                                }
                            }
                            let merged_child_splits = merge_splits(&child_splits, max_len);
                            add_splits(&mut acc, &merged_child_splits.clone(), max_len);
                            acc
                        })
                }
            } else {
                let child_splits = self.children
                    .iter()
                    .map(|child| child.get_node_splits(max_tokens, merge_level))
                    .flatten()
                    .collect();
                let res = attach_pattern_nodes(&child_splits, usize::MAX);
                res
            }
        }
    }
}
