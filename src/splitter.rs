use aho_corasick::Span;

use crate::pattern_search::{find_pattern, SearchResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitNode<'a> {
    pub pattern_id: usize,
    pub data_span: Span,
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
                result.push_str(&format!(
                    "{} orig: {:?}\n",
                    indent_str,
                    self.search_result.reconstruct()
                ));
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
                result.push_str(&format!(
                    "{} orig: {:?}\n",
                    indent_str,
                    self.search_result.reconstruct()
                ));
            }
            result
        }
    }
    pub fn to_string(&self, reconstruct: bool) -> String {
        self.to_string_with_indent(0, reconstruct)
    }

    pub fn reconstruct(&self) -> String {
        self.search_result.reconstruct()
    }

    pub fn merge(&self, merge_level: usize) -> Vec<Span> {
        let mut result = Vec::new();
        if self.pattern_id > merge_level {
            result.push(self.search_result.full_span());
        } else {
            for child in &self.children {
                result.extend(child.merge(merge_level));
            }
        }
        result
    }
}

pub fn split<'a>(
    data: &'a [u8],
    data_span: Span,
    patterns: Vec<&'a str>,
    pattern_id: usize,
) -> SplitNode<'a> {
    let pattern = patterns[pattern_id];
    let search_result = find_pattern(pattern, data, data_span);

    if search_result.matched {
        let children: Vec<_> = search_result
            .splits
            .iter()
            .map(|search_split| {
                if pattern_id + 1 < patterns.len() {
                    split(data, search_split.span, patterns.clone(), pattern_id + 1)
                } else {
                    SplitNode {
                        pattern_id: pattern_id + 1,
                        data_span: search_split.span,
                        search_result: SearchResult::from(search_split.clone()),
                        children: Vec::new(),
                    }
                }
            })
            .collect();

        SplitNode {
            pattern_id,
            data_span,
            search_result,
            children,
        }
    } else {
        if pattern_id + 1 < patterns.len() {
            split(data, data_span, patterns, pattern_id + 1)
        } else {
            SplitNode {
                pattern_id,
                data_span,
                search_result,
                children: Vec::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;

    #[test]
    fn single_level_split() {
        let data = "Hello, you all!\n How are you? \n".as_bytes();
        let patterns = vec!["\n\n", "\n"];
        let tree = split(
            data,
            Span {
                start: 0,
                end: data.len(),
            },
            patterns,
            0,
        );
        println!("{:?}", tree);
        println!("{}", tree.to_string(true));
        let reconsted = tree.reconstruct();
        assert_eq!(from_utf8(data).unwrap(), reconsted);
    }

    #[test]
    fn multi_level_split() {
        let data = "\n\n\
        \n\n\
        \n\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\n\
        Another heuristic for.\n\
        \n\n\
        \n\n\
        \n\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let patterns = vec!["\n\n", "\n", "."];
        //let patterns = vec!["\n\n".to_string()];
        //let patterns = vec!["\n\n".to_string(), "\n".to_string()];
        let tree = split(
            data,
            Span {
                start: 0,
                end: data.len(),
            },
            patterns,
            0,
        );
        println!("{}", tree.to_string(true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());

        let merged = tree.merge(1);
        for span in merged {
            println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
        }
    }

    #[test]
    fn first_pattern_no_match_split() {
        let data = "\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\
        Another heuristic for.\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n"
            .as_bytes();

        let patterns = vec!["\n\n", "\n", "."];
        //let patterns = vec!["\n\n".to_string()];
        //let patterns = vec!["\n\n".to_string(), "\n".to_string()];
        let tree = split(
            data,
            Span {
                start: 0,
                end: data.len(),
            },
            patterns,
            0,
        );
        println!("{}", tree.to_string(true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct());

        let merged = tree.merge(1);
        for span in merged {
            println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
        }
    }
}
