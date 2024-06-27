use crate::pattern_search::pattern_searcher::PatternSearcher;
use crate::pattern_search::search_result::SearchResult;
use crate::pattern_search::search_split::SearchSplit;
use aho_corasick::Span;

pub struct ParseTree<'a> {
    parse_nodes: Option<Vec<ParseNode<'a>>>,
    search_result: Option<SearchResult<'a>>,
}

pub struct ParseNode<'a> {
    parse_tree: Option<ParseTree<'a>>,
    level: Option<usize>,
    search_split: Option<SearchSplit<'a>>,
}

pub fn parse_tree(searchers: Vec<PatternSearcher>, data: &[u8]) -> ParseTree {
    for (pattern_level, searcher) in searchers.iter().enumerate() {}

    ParseTree {
        parse_nodes: None,
        search_result: None,
    }
}

pub fn parse_level<'a>(
    parse_tree: &'a mut ParseTree<'a>,
    searcher: &'a PatternSearcher,
    pattern_level: usize,
    data_span: Span,
    data: &'a [u8],
) {
    let mut parse_nodes = Vec::new();
    let search_res = searcher.find_pattern(data, data_span.clone());
    if search_res.matched {
        search_res.splits.iter().for_each(|search_split| {
            let parse_node = ParseNode {
                parse_tree: None,
                level: Some(pattern_level),
                search_split: Some(search_split.clone()),
            };
            parse_nodes.push(parse_node);
        });
    }
    parse_tree.parse_nodes = if parse_nodes.is_empty() {
        None
    } else {
        Some(parse_nodes)
    };
    parse_tree.search_result = Some(search_res);
}
