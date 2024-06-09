use std::str::from_utf8;

use aho_corasick::Span;
use rayon::prelude::*;

use crate::common::{chunk_spans, span};
use crate::pattern_search::{PatternSearcher, SearchSplit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitNode {
    pub split_span: Span,
    pub pattern_id: usize,
    pub children: Vec<SplitNode>,
}

impl SplitNode {
    pub fn to_string_with_indent(&self, data: &[u8], indent: usize, reconstruct: bool) -> String {
        let indent_str = " ".repeat(indent);

        if !self.children.is_empty() {
            let mut result = format!(
                "{}Node:\n{} pattern_id: {} \n{} len: {} \n{} children:\n",
                indent_str,
                indent_str,
                self.pattern_id,
                indent_str,
                self.split_span.len(),
                indent_str
            );
            self.children.iter().for_each(|child| {
                result.push_str(&child.to_string_with_indent(data, indent + 4, reconstruct));
            });
            if reconstruct {
                result.push_str(&format!("{} orig: {:?}\n", indent_str, self.reconstruct(data)));
            }
            result
        } else {
            let mut result = format!(
                "{}Leaf: \n{} pattern_id: {} \n{} len: {} \n",
                indent_str,
                indent_str,
                self.pattern_id,
                indent_str,
                self.split_span.len(),
            );
            if reconstruct {
                result.push_str(&format!("{} orig: {:?}\n", indent_str, self.reconstruct(data)));
            }
            result
        }
    }
    pub fn to_string(&self, data: &[u8], reconstruct: bool) -> String {
        self.to_string_with_indent(data, 0, reconstruct)
    }

    pub fn reconstruct(&self, data: &[u8]) -> String {
        from_utf8(&data[self.split_span.start..self.split_span.end])
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

    //FIXME: This needs more testing, and see if it can be combined with merge_splits
    pub fn leaf_level(&self) -> Option<usize> {
        if self.children.is_empty() {
            Some(self.pattern_id)
        } else {
            let leaf_found = self.children.iter().find(|child| child.children.is_empty());
            if let Some(leaf) = leaf_found {
                Some(leaf.pattern_id)
            } else {
                self.children.iter().find(|child| child.leaf_level().is_some()).map(|leaf| leaf.pattern_id)
            }
        }
    }

    pub fn all_leaves(&self) -> Vec<Span> {
        if self.children.is_empty() {
            vec![self.split_span]
        } else {
            self.children
                .iter()
                .flat_map(|child| child.all_leaves())
                .collect()
        }
    }
}

pub struct SplitterConfig<'a> {
    pub data: &'a [u8],
    pub patterns: &'a Vec<Vec<&'a str>>,
    pub searchers: &'a Vec<PatternSearcher<'a>>,
    pub max_len: Option<usize>,
}

pub fn split(config: &SplitterConfig,
             search_span: Span,
             pattern_id: usize,
             split_span: Span,
             parallel: Option<bool>,
) -> SplitNode {
    let max_len_check = config.max_len.is_some();
    let max_len_value = config.max_len.unwrap_or(0);
    if max_len_check && split_span.len() <= max_len_value {
        return SplitNode {
            pattern_id,
            split_span,
            children: Vec::new(),
        };
    }

    let search_result = config.searchers[pattern_id].find_pattern(config.data, search_span);

    if search_result.matched {
        let parallelize = parallel.unwrap_or_else(|| false);

        let children: Vec<_> = if parallelize {
            search_result
                .splits
                .par_iter()
                .flat_map(|search_split| {
                    sub_split(config, pattern_id, search_split, None)
                })
                .collect()
        } else {
            search_result
                .splits
                .iter()
                .flat_map(|search_split| {
                    sub_split(config, pattern_id, search_split, None)
                })
                .collect()
        };

        SplitNode {
            pattern_id,
            split_span,
            children,
        }
    } else {
        if pattern_id + 1 < config.patterns.len() && !search_span.is_empty() {
            split(config,
                  search_span,
                  pattern_id + 1,
                  split_span, parallel,
            )
        } else {
            if max_len_check {
                let children = split_chunk(pattern_id, split_span, max_len_value);
                SplitNode {
                    pattern_id,
                    split_span,
                    children,
                }
            } else {
                SplitNode {
                    pattern_id,
                    split_span,
                    children: Vec::new(),
                }
            }
        }
    }
}


fn sub_split(config: &SplitterConfig,
             pattern_id: usize,
             search_split: &SearchSplit,
             parallel: Option<bool>,
) -> Vec<SplitNode> {
    let max_len_check = config.max_len.is_some();
    let max_len_value = config.max_len.unwrap_or(0);
    if pattern_id + 1 < config.patterns.len() && !search_split.span.is_empty() {
        //FIXME: add split pattern as child node
        let mut child_nodes = Vec::new();

        let child_node = split(config,
                               search_split.span,
                               pattern_id + 1,
                               search_split.span,
                               parallel,
        );
        child_nodes.push(child_node);

        if search_split.stride > 0 {
            let child_node = SplitNode {
                pattern_id: pattern_id + 1,
                split_span: span(
                    search_split.span.end,
                    search_split.span.end + search_split.pattern.len(),
                ),
                children: Vec::new(),
            };
            child_nodes.push(child_node);
        }
        child_nodes
    } else {
        if max_len_check {
            if search_split.full_span().len() <= max_len_value {
                let child_node = SplitNode {
                    pattern_id: pattern_id + 1,
                    split_span: search_split.full_span(),
                    children: Vec::new(),
                };
                vec![child_node]
            } else {
                split_chunk(
                    pattern_id + 1,
                    search_split.full_span(),
                    max_len_value,
                )
            }
        } else {
            let child_node = SplitNode {
                pattern_id: pattern_id + 1,
                split_span: search_split.full_span(),
                children: Vec::new(),
            };
            vec![child_node]
        }
    }
}

pub fn split_chunk(pattern_id: usize,
                   split_span: Span,
                   max_len_value: usize,
) -> Vec<SplitNode> {
    let mut result = Vec::new();
    let mut start = split_span.start;
    let end = split_span.end;
    while start < end {
        let chunk_end = if end - start > max_len_value {
            start + max_len_value
        } else {
            end
        };
        let chunk_span = Span::from(start..chunk_end);
        result.push(SplitNode {
            pattern_id,
            split_span: chunk_span,
            children: Vec::new(),
        });
        start = chunk_end;
    }
    result
}
