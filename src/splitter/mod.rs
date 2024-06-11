use std::sync::Arc;

use aho_corasick::Span;
use rayon::prelude::*;

use crate::common::span;
use crate::encodings::Tokenize;
use crate::pattern_search::search_split::SearchSplit;
use crate::splitter::split_encoding::SplitEncoding;
use crate::splitter::split_node::SplitNode;
use crate::splitter::splitter_config::SplitterConfig;

mod split_encoding;
pub mod split_node;
pub mod splitter_config;
mod tests;
mod utils;

pub struct Splitter<'a, T: Tokenize + Sync> {
    config: &'a SplitterConfig<'a, T>,
    search_data_span: Span,
    pattern_id: usize,
    split_data_span: Span,
    parallel: Option<bool>,
    split_encoding: Option<Arc<SplitEncoding>>,
    split_tokens_span: Option<Span>,
}

impl<'a, T: Tokenize + Sync> Splitter<'a, T> {
    pub fn new(
        config: &'a SplitterConfig<'a, T>,
        search_data_span: Span,
        pattern_id: usize,
        split_data_span: Span,
        parallel: Option<bool>,
        split_encoding: Option<Arc<SplitEncoding>>,
        split_tokens_span: Option<Span>,
    ) -> Self {
        Splitter {
            config,
            search_data_span,
            pattern_id,
            split_data_span,
            parallel,
            split_encoding,
            split_tokens_span,
        }
    }

    pub fn to_next_splitter(&self) -> Self {
        Splitter::new(
            self.config,
            self.search_data_span,
            self.pattern_id + 1,
            self.split_data_span,
            self.parallel,
            self.split_encoding.clone(),
            self.split_tokens_span.clone(),
        )
    }

    pub fn to_sub_splitter(
        &self,
        search_split: &SearchSplit,
        pattern_id: usize,
        parallel: Option<bool>,
        split_encoding: Option<Arc<SplitEncoding>>,
        split_tokens_span: Option<Span>,
    ) -> Self {
        Splitter::new(
            self.config,
            search_split.span,
            pattern_id,
            search_split.span,
            parallel,
            split_encoding,
            split_tokens_span,
        )
    }

    pub fn split(&self) -> SplitNode {
        if self.lt_max_len(self.split_data_span) {
            let (new_split_encoding, new_split_tokens_span) = self.config.get_split_encoding(
                &self.split_data_span,
                &self.split_encoding,
                &self.split_tokens_span,
            );

            return SplitNode::new(
                self.pattern_id,
                self.split_data_span,
                Vec::new(),
                new_split_encoding.clone(),
                new_split_tokens_span.clone(),
            );
        }

        let search_result = self.config.searchers[self.pattern_id]
            .find_pattern(self.config.data, self.search_data_span);

        if search_result.matched {
            let parallelize = self.parallel.unwrap_or_else(|| false);

            let children: Vec<_> = if parallelize {
                search_result
                    .splits
                    .par_iter()
                    .flat_map(|search_split| {
                        let (new_split_encoding, new_split_tokens_span) =
                            self.config.get_split_encoding(
                                &search_split.span,
                                &self.split_encoding,
                                &self.split_tokens_span,
                            );

                        let sub_splitter = self.to_sub_splitter(
                            search_split,
                            self.pattern_id,
                            None,
                            new_split_encoding,
                            new_split_tokens_span,
                        );

                        sub_splitter.sub_split(search_split)
                    })
                    .collect()
            } else {
                //if split encoding is not present, then split need to be encoded
                let do_encode = self.split_encoding.is_none();
                // encode the split if needed
                if do_encode {
                    // do encoding
                    search_result
                        .splits
                        .iter()
                        .flat_map(|search_split| {
                            let (new_split_encoding, new_split_tokens_span) =
                                self.config.get_split_encoding(
                                    &search_split.span,
                                    &self.split_encoding,
                                    &self.split_tokens_span,
                                );

                            let sub_splitter = self.to_sub_splitter(
                                search_split,
                                self.pattern_id,
                                None,
                                new_split_encoding,
                                new_split_tokens_span,
                            );

                            sub_splitter.sub_split(search_split)
                        })
                        .collect()
                } else {
                    // no encoding needed
                    // using the existing encoding to split the tokens
                    search_result
                        .splits
                        .iter()
                        .scan(
                            self.split_tokens_span.clone().unwrap(),
                            |tokens_span, search_split| {
                                //FIXME: there must be some issue with the offsets
                                let sub_tokens_span = self
                                    .split_encoding
                                    .as_ref()
                                    .unwrap()
                                    .split_tokens_span(search_split.span, tokens_span.clone())
                                    .unwrap_or(*tokens_span);

                                *tokens_span = if sub_tokens_span == *tokens_span {
                                    *tokens_span
                                } else {
                                    span(sub_tokens_span.end, tokens_span.end)
                                };

                                let sub_splitter = self.to_sub_splitter(
                                    search_split,
                                    self.pattern_id,
                                    None,
                                    self.split_encoding.clone(),
                                    Some(sub_tokens_span),
                                );

                                let children = sub_splitter.sub_split(search_split);

                                Some(children)
                            },
                        )
                        .flat_map(|x| x)
                        .collect()
                }
            };

            SplitNode::new(
                self.pattern_id,
                self.split_data_span,
                children,
                self.split_encoding.clone(),
                self.split_tokens_span,
            )
        } else {
            if self.pattern_id + 1 < self.config.patterns.len() && !self.search_data_span.is_empty()
            {
                let new_splitter = self.to_next_splitter();
                new_splitter.split()
            } else {
                let (new_split_encoding, new_split_tokens_span) = self.config.get_split_encoding(
                    &self.split_data_span,
                    &self.split_encoding,
                    &self.split_tokens_span,
                );

                if !self.lt_max_len(self.split_data_span) {
                    let children = self.chunk_splits(self.pattern_id, self.split_data_span);

                    SplitNode::new(
                        self.pattern_id,
                        self.split_data_span,
                        children,
                        new_split_encoding,
                        new_split_tokens_span,
                    )
                } else {
                    SplitNode::new(
                        self.pattern_id,
                        self.split_data_span,
                        Vec::new(),
                        new_split_encoding,
                        new_split_tokens_span,
                    )
                }
            }
        }
    }

    fn sub_split(&self, search_split: &SearchSplit) -> Vec<SplitNode> {
        if self.pattern_id + 1 < self.config.patterns.len() && !search_split.span.is_empty() {
            let mut child_nodes = Vec::new();

            let splitter = self.to_sub_splitter(
                search_split,
                self.pattern_id + 1,
                self.parallel,
                self.split_encoding.clone(),
                self.split_tokens_span,
            );

            let child_node = splitter.split();
            child_nodes.push(child_node);

            if search_split.stride > 0 {

                let child_node = SplitNode::new(
                    self.pattern_id + 1,
                    span(
                        search_split.span.end,
                        search_split.span.end + search_split.pattern.len(),
                    ),
                    Vec::new(),
                    //FIXME: this will give an issue with encoding
                    None,
                    None,
                );
                child_nodes.push(child_node);
            }
            child_nodes
        } else {
            if self.lt_max_len(search_split.full_span()) {
                let child_node = SplitNode::new(
                    self.pattern_id + 1,
                    search_split.full_span(),
                    Vec::new(),
                    self.split_encoding.clone(),
                    self.split_tokens_span,
                );
                vec![child_node]
            } else {
                self.chunk_splits(self.pattern_id + 1, search_split.full_span())
            }
        }
    }

    pub fn chunk_splits(&self, pattern_id: usize, data_span: Span) -> Vec<SplitNode> {
        let max_len_val = self.config.max_len.unwrap_or(0);
        if max_len_val == 0 {
            return vec![SplitNode::new(
                pattern_id,
                data_span,
                Vec::new(),
                self.split_encoding.clone(),
                self.split_tokens_span,
            )];
        }

        if self.split_encoding.is_some() {
            let split_spans = utils::chunk_tokens_len(
                self.split_encoding
                    .as_ref()
                    .unwrap()
                    .encoding
                    .get_word_ids(),
                self.split_tokens_span.unwrap(),
                max_len_val,
            );

            let encoding_offset= self.split_encoding.as_ref().unwrap().encoding_data_span.start;

            let tokens_offsets = self
                .split_encoding
                .clone()
                .unwrap()
                .encoding
                .to_data_offsets_new(
                    split_spans.clone(),
                    encoding_offset,
                );

            split_spans
                .iter()
                .zip(tokens_offsets)
                .map(|(tokens_span, tokens_data_span)| {
                    SplitNode::new(
                        pattern_id,
                        tokens_data_span,
                        Vec::new(),
                        self.split_encoding.clone(),
                        Some(tokens_span.clone()),
                    )
                })
                .collect()
        } else {
            utils::split_data_len(pattern_id, data_span, max_len_val)
        }
    }

    pub fn lt_max_len(&self, split_span: Span) -> bool {
        let check_max_len = self.config.max_len.is_some();

        if check_max_len {
            let max_len_val = self.config.max_len.unwrap();
            if self.split_encoding.is_some() {
                self.split_tokens_span.unwrap().len() <= max_len_val
            } else {
                // if tokenizer is present, then don't need to check the length just yet
                // as the data will be tokenized further down the line
                if self.config.tokenizer.is_some() {
                    false
                } else {
                    split_span.len() <= max_len_val
                }
            }
        } else {
            false
        }
    }
}
