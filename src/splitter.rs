use aho_corasick::Span;
use std::str::from_utf8;

use crate::pattern_search::{find_pattern, SearchResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitNode<'a> {
    pub split_span: Span,
    pub data: &'a [u8],
    pub pattern_id: usize,
    pub search_result: SearchResult<'a>,
    pub children: Vec<SplitNode<'a>>,
}

impl SplitNode<'_> {
    pub fn to_string_with_indent(&self, indent: usize, reconstruct: bool) -> String {
        let indent_str = " ".repeat(indent);

        if !self.children.is_empty() {
            let mut result = format!(
                "{}Node:\n{} pattern_id: {} \n{} children:\n",
                indent_str, indent_str, self.pattern_id, indent_str
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
                "{}Leaf: \n{} pattern_id: {},\n{} data: {:?} \n",
                indent_str,
                indent_str,
                self.pattern_id,
                indent_str,
                self.search_result.data()
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

    pub fn merge(&self, merge_level: usize, filter_empty: bool) -> Vec<Span> {
        let mut result = Vec::new();
        let filter = filter_empty && self.search_result.is_empty();
        if !filter && self.pattern_id > merge_level {
            result.push(self.split_span);
        } else {
            for child in &self.children {
                result.extend(child.merge(merge_level, filter_empty));
            }
        }
        result
    }
}

pub fn split<'a>(
    data: &'a [u8],
    search_span: Span,
    patterns: Vec<&'a str>,
    pattern_id: usize,
    split_span: Span,
) -> SplitNode<'a> {
    let pattern = patterns[pattern_id];
    let search_result = find_pattern(pattern, data, search_span);

    if search_result.matched {
        let children: Vec<_> = search_result
            .splits
            .iter()
            .map(|search_split| {
                if pattern_id + 1 < patterns.len() && !search_split.is_empty() {
                    split(
                        data,
                        search_split.span,
                        patterns.clone(),
                        pattern_id + 1,
                        search_split.full_span(),
                    )
                } else {
                    SplitNode {
                        data,
                        pattern_id: pattern_id + 1,
                        split_span: search_split.full_span(),
                        search_result: SearchResult::from(search_split.clone()),
                        children: Vec::new(),
                    }
                }
            })
            .collect();

        SplitNode {
            data,
            pattern_id,
            split_span,
            search_result,
            children,
        }
    } else {
        if pattern_id + 1 < patterns.len() && !search_span.is_empty() {
            split(data, search_span, patterns, pattern_id + 1, split_span)
        } else {
            SplitNode {
                data,
                pattern_id,
                split_span,
                search_result,
                children: Vec::new(),
            }
        }
    }
}
