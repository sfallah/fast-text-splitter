use std::str::from_utf8;

use aho_corasick::Span;

use crate::pattern_search::{PatternSearcher, SearchResult, SearchSplit};

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
                "{}Node:\n{} pattern_id: {} \n{} len: {} \n{} children:\n",
                indent_str, indent_str, self.pattern_id, indent_str, self.split_span.len(), indent_str
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
                "{}Leaf: \n{} pattern_id: {} \n{} len: {} \n{} data: {:?} \n",
                indent_str,
                indent_str,
                self.pattern_id,
                indent_str,
                self.split_span.len(),
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
    pub fn all_leaves(&self) -> Vec<Span> {
        if self.children.is_empty() {
            vec![self.split_span]
        } else {
            self.children.iter().flat_map(|child| child.all_leaves()).collect()
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
) -> SplitNode<'a> {
    let max_len_check = max_len.is_some();
    let max_len_value = max_len.unwrap_or(0);
    if max_len_check && split_span.len() <= max_len_value {
        return SplitNode {
            data,
            pattern_id,
            split_span,
            search_result: SearchResult::from(SearchSplit::new(search_span, data, "", 0)),
            children: Vec::new(),
        };
    }

    let search_result = searchers[pattern_id].find_pattern(data, search_span);

    if search_result.matched {
        let children: Vec<_> = search_result
            .splits
            .iter()
            .flat_map(|search_split| {
                if pattern_id + 1 < patterns.len() && !search_split.is_empty() {
                    let child_node = split(
                        data,
                        search_split.span,
                        patterns,
                        pattern_id + 1,
                        search_split.full_span(),
                        searchers,
                        max_len,
                    );
                    vec![child_node]
                } else {
                    if max_len_check {
                        if search_split.len() <= max_len_value {
                            let child_node = SplitNode {
                                data,
                                pattern_id: pattern_id + 1,
                                split_span: search_split.full_span(),
                                search_result: SearchResult::from(search_split.clone()),
                                children: Vec::new(),
                            };
                            vec![child_node]
                        } else {
                            split_chunk(data, pattern_id + 1, search_split.full_span(), max_len_value)
                        }
                    } else {
                        let child_node = SplitNode {
                            data,
                            pattern_id: pattern_id + 1,
                            split_span: search_split.full_span(),
                            search_result: SearchResult::from(search_split.clone()),
                            children: Vec::new(),
                        };
                        vec![child_node]
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
            split(
                data,
                search_span,
                patterns,
                pattern_id + 1,
                split_span,
                searchers,
                max_len,
            )
        } else {
            if max_len_check {
                let children = split_chunk(data, pattern_id, split_span, max_len_value);
                SplitNode {
                    data,
                    pattern_id,
                    split_span,
                    search_result,
                    children,
                }
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
        let search_result = SearchResult::from(SearchSplit::new(chunk_span, data, "", 0));
        result.push(SplitNode {
            data,
            pattern_id,
            split_span: chunk_span,
            search_result,
            children: Vec::new(),
        });
        start = chunk_end;
    }
    result
}
