use std::str::from_utf8;

use aho_corasick::Span;

use crate::common::span;
use crate::pattern_search::{PatternSearcher, SearchSplit};
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitNode<'a> {
    pub split_span: Span,
    pub data: &'a [u8],
    pub pattern_id: usize,
    pub children: Vec<SplitNode<'a>>,
}

impl SplitNode<'_> {
    pub fn to_string_with_indent(&self, indent: usize, reconstruct: bool) -> String {
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
                result.push_str(&child.to_string_with_indent(indent + 4, reconstruct));
            });
            if reconstruct {
                result.push_str(&format!("{} orig: {:?}\n", indent_str, self.reconstruct()));
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
                result.push_str(&format!("{} orig: {:?}\n", indent_str, self.reconstruct()));
            }
            result
        }
    }
    pub fn to_string(&self, reconstruct: bool) -> String {
        self.to_string_with_indent(0, reconstruct)
    }

    pub fn reconstruct(&self) -> String {
        from_utf8(&self.data[self.split_span.start..self.split_span.end])
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

pub fn split<'a>(
    data: &'a [u8],
    search_span: Span,
    patterns: &Vec<Vec<&'a str>>,
    pattern_id: usize,
    split_span: Span,
    searchers: &'a Vec<PatternSearcher<'a>>,
    max_len: Option<usize>,
    parallel_level: Option<usize>,
) -> SplitNode<'a> {
    let max_len_check = max_len.is_some();
    let max_len_value = max_len.unwrap_or(0);
    if max_len_check && split_span.len() <= max_len_value {
        return SplitNode {
            data,
            pattern_id,
            split_span,
            children: Vec::new(),
        };
    }

    let search_result = searchers[pattern_id].find_pattern(data, search_span);

    if search_result.matched {
        let parallelize = match parallel_level {
            Some(level) => if level > 0 { false } else { true },
            None => false,
        };

        let children: Vec<_> = if parallelize {
            search_result
                .splits
                .par_iter()
                .flat_map(|search_split| {
                    sub_split(data, patterns, pattern_id, searchers, max_len, search_split, Some(pattern_id + 1))
                })
                .collect()
        } else {
            search_result
                .splits
                .iter()
                .flat_map(|search_split| {
                    sub_split(data, patterns, pattern_id, searchers, max_len, search_split, parallel_level)
                })
                .collect()
        };

        SplitNode {
            data,
            pattern_id,
            split_span,
            children,
        }
    } else {
        if pattern_id + 1 < patterns.len() && !search_span.is_empty() {
            split(
                data,
                search_span,
                patterns,
                pattern_id + 1,
                split_span,
                searchers,
                max_len,
                parallel_level,
            )
        } else {
            if max_len_check {
                let children = split_chunk(data, pattern_id, split_span, max_len_value);
                SplitNode {
                    data,
                    pattern_id,
                    split_span,
                    children,
                }
            } else {
                SplitNode {
                    data,
                    pattern_id,
                    split_span,
                    children: Vec::new(),
                }
            }
        }
    }
}

fn sub_split<'a>(
    data: &'a [u8],
    patterns: &Vec<Vec<&'a str>>,
    pattern_id: usize,
    searchers: &'a Vec<PatternSearcher<'a>>,
    max_len: Option<usize>,
    search_split: &SearchSplit,
    parallel_level: Option<usize>,
) -> Vec<SplitNode<'a>> {
    let max_len_check = max_len.is_some();
    let max_len_value = max_len.unwrap_or(0);
    if pattern_id + 1 < patterns.len() && !search_split.span.is_empty() {
        //FIXME: add split pattern as child node
        let mut child_nodes = Vec::new();

        let child_node = split(
            data,
            search_split.span,
            patterns,
            pattern_id + 1,
            search_split.span,
            searchers,
            max_len,
            parallel_level,
        );
        child_nodes.push(child_node);

        if search_split.stride > 0 {
            let child_node = SplitNode {
                data,
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
                    data,
                    pattern_id: pattern_id + 1,
                    split_span: search_split.full_span(),
                    children: Vec::new(),
                };
                vec![child_node]
            } else {
                split_chunk(
                    data,
                    pattern_id + 1,
                    search_split.full_span(),
                    max_len_value,
                )
            }
        } else {
            let child_node = SplitNode {
                data,
                pattern_id: pattern_id + 1,
                split_span: search_split.full_span(),
                children: Vec::new(),
            };
            vec![child_node]
        }
    }
}

pub fn split_chunk(
    data: &[u8],
    pattern_id: usize,
    split_span: Span,
    max_len: usize,
) -> Vec<SplitNode> {
    let mut result = Vec::new();
    let mut start = split_span.start;
    let end = split_span.end;
    while start < end {
        let chunk_end = if end - start > max_len {
            start + max_len
        } else {
            end
        };
        let chunk_span = Span::from(start..chunk_end);
        result.push(SplitNode {
            data,
            pattern_id,
            split_span: chunk_span,
            children: Vec::new(),
        });
        start = chunk_end;
    }
    result
}

pub fn chunk_spans(spans: &Vec<Span>, max_len: usize) -> Vec<Span> {
    if spans.is_empty() {
        return Vec::new();
    }

    let mut cur_split = spans[0];
    let mut cur_len: usize = cur_split.len();
    assert!(
        cur_len <= max_len,
        "First Span: {:?} with len: {} is larger than max_len: {}",
        cur_split,
        cur_split.len(),
        max_len
    );
    if spans.len() == 1 {
        return vec![cur_split];
    }

    let mut chunked_splits = Vec::new();

    for (idx, leaf) in spans.iter().enumerate().skip(1) {
        assert!(
            cur_split.len() <= max_len,
            "Current Span: {:?} with len: {} is larger than max_len: {}",
            cur_split,
            cur_split.len(),
            max_len
        );

        assert_eq!(
            cur_split.end, leaf.start,
            "Next Span: {:?} doesn't continue the Current Span: {:?}",
            leaf, cur_split
        );

        if cur_len + leaf.len() > max_len {
            chunked_splits.push(cur_split);
            cur_split = leaf.clone();
            cur_len = leaf.len();
        } else {
            cur_split = span(cur_split.start, leaf.end);
            cur_len += leaf.len();
        }
        if idx == spans.len() - 1 {
            chunked_splits.push(cur_split);
        }
    }

    chunked_splits
}
