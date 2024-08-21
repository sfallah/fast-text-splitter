#[cfg(test)]
mod tests {
    use aho_corasick::Span;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use rayon::prelude::*;
    use std::io::Write;
    use std::str::from_utf8;
    use std::{fs, io};

    use crate::config::SplitterLiteConfig;
    use crate::encodings::{NoneTokenizer, Tokenize};
    use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
    use crate::normalizer::TextNormalizer;
    use crate::pattern_search::pattern_searcher::PatternSearcher;
    use crate::splitter::split_node::utils::SplitResultLite;
    use crate::splitter::split_node::visualization::term_tree;
    use crate::splitter::{Splitter, SplitterConfig};
    use crate::ws_tokenizer::WSTokenizer;

    fn list_text_files(dir: &str) -> io::Result<Vec<String>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                files.push(String::from(path.to_str().unwrap()));
            }
        }

        Ok(files)
    }

    fn split_file<'a>(
        data_path: &str,
        patterns: Vec<Vec<String>>,
        searchers: &Vec<PatternSearcher>,
        max_len: usize,
        merge_level: usize,
        _print: bool,
        check_splits: bool,
        parallel: bool,
        sub_splits: bool,
    ) -> (usize, Vec<SplitResultLite>) {
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();
        let patterns_len = patterns.len();

        let origin_dta_span = Span {
            start: 0,
            end: data.len(),
        };

        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, Some(data.len()), false).unwrap(),
        };

        let max_len = Some(max_len);
        let config = SplitterConfig::<HFTokenizer> {
            data,
            searchers,
            max_len,
            merge_level: None,
            tokenizer: Some(&hf_tokenizer),
            patterns_len,
        };

        let splitter = Splitter::new(
            &config,
            origin_dta_span,
            0,
            origin_dta_span,
            Some(parallel),
            None,
            None,
        );
        let tree = splitter.split();

        let split_results = tree.get_results_lite(max_len, Some(merge_level), data);
        println!("Number of Split Results: {:?}", split_results.len());

        if _print {
            for res in split_results.iter() {
                println!("{:?}", res.split_string);
                println!("Data Len: {:?}", res.split_string.len());
                println!("Tokens Len: {:?}", res.tokens.len());
            }
        }

        let total_text_len: usize = split_results.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_text_len, "File Path: {:?}", data_path);

        if check_splits {
            for res in split_results.iter() {
                println!("{:?}", res.split_string);

                let hf_encoded = hf_tokenizer.encode(&res.split_string).unwrap();
                println!("HF Tokens len: {:?}", hf_encoded.len());

                assert_eq!(
                    hf_encoded.len(),
                    res.tokens.len(),
                    "Failed Split: {:?}\n Tokens-Result: {:?}",
                    res.split_string,
                    res,
                );
            }
        }

        let total_tokens: usize = split_results.iter().map(|res| res.tokens.len()).sum();
        println!("Total Tokens: {:?}", total_tokens);

        let encodings = hf_tokenizer.encode(from_utf8(data).unwrap()).unwrap();
        assert_eq!(total_tokens, encodings.len(), "File Path: {:?}", data_path);

        if sub_splits {
            let sub_splitter_config =
                SplitterLiteConfig::new_hf(patterns, None, Some(3), true, None);
            split_results.par_iter().for_each(|split| {
                let sub_splits = sub_splitter_config.hf_splits(split.split_string.as_bytes());
                let sub_total_len: usize = sub_splits
                    .iter()
                    .map(|sub_split| sub_split.split_string.len())
                    .sum();
                assert_eq!(split.split_string.len(), sub_total_len);
                let sub_total_tokens: usize = sub_splits
                    .iter()
                    .map(|sub_split| sub_split.tokens.len())
                    .sum();
                assert_eq!(split.tokens.len(), sub_total_tokens);
            })
        }

        (data.len(), split_results)
    }

    #[test]
    fn single_level_split() {
        let data = "Hello, you all.\n How are you.\n".as_bytes();
        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len: Some(10),
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        println!("{}", tree.to_string(data, true));
        let reconsted = tree.reconstruct(data);
        assert_eq!(from_utf8(data).unwrap(), reconsted);

        let lite_results = tree.get_results_lite(Some(20), None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("text: {:?}", res.split_string);
            println!("len: {:?}", res.split_string.len());
        }
        assert_eq!(lite_results.len(), 2);
        let total_len: usize = lite_results.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_len);
    }

    #[test]
    fn multi_level_none_tokenizer_split() {
        let _data_raw = "\n\n\
        \n\n\
        \n\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\n\
        Another heuristic for.\n\n\
        \n\n\
        \n\n\
        \nFinding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len: Some(32),
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));

        let lite_results = tree.get_results_lite(Some(32), None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("text: {:?}", res.split_string);
            println!("len: {:?}", res.split_string.len());
        }
    }
    #[test]
    fn no_pattern_test() {
        let data = "this is a sentence".as_bytes();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, None, false).unwrap(),
        };
        let max_len = Some(512);

        let config = SplitterConfig::<HFTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&hf_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));

        let lite_results = tree.get_results_lite(Some(512), None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn long_no_match_test() -> tokenizers::Result<()> {
        //let data_path = "tests/test_data/en_long_sentence.txt";
        let data_path = "tests/test_data/en_long_paragraphs.txt";
        let binding = fs::read(data_path).unwrap();
        let data = binding.as_slice();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let max_len = Some(128);
        let splitter_config = SplitterLiteConfig::new_hf(patterns, max_len, None, true, None);
        let splits = splitter_config.hf_splits(data);

        let hf_tokenizer = init_tokenizer(None, None, false).unwrap();

        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_len);
        let hf_encoding = hf_tokenizer
            .encode(from_utf8(data).unwrap(), false)
            .unwrap();
        let hf_tokens = hf_encoding.get_ids();
        let total_tokens: usize = splits.iter().map(|res| res.tokens.len()).sum();
        assert_eq!(hf_tokens.len(), total_tokens);

        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.tokens.len());
            let hf_encoded = hf_tokenizer.encode(split.split_string.clone(), false)?;
            assert_eq!(hf_encoded.len(), split.tokens.len());
            assert!(!split.tokens.is_empty());
            assert!(split.tokens.len() <= max_len.unwrap());
            assert_eq!(hf_encoded.get_ids(), split.tokens);
        }
        Ok(())
    }

    #[test]
    fn multi_level_ws_tokenizer_split() {
        let _data_raw = "\n\n\
        \n\n\
        \n\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\n\
        Another heuristic for.\n\n\
        \n\n\
        \n\n\
        \nFinding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let ws_tokenizer = WSTokenizer { ascii: false };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let max_len = Some(16);
        let config = SplitterConfig::<WSTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&ws_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));

        let lite_results = tree.get_results_lite(max_len, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn merge_enc_result_test() {
        let _data_raw = "\n\n\
        \n\n\
        \n\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\n\
        Another heuristic for.\n\n\
        \n\n\
        \n\n\
        \nFinding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let tokenizer = init_tokenizer(None, None, false).unwrap();

        let hf_tokenizer = HFTokenizer { tokenizer };

        let max_len = Some(4);

        let config = SplitterConfig::<HFTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&hf_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        let lite_results = tree.get_results_lite(Some(16), None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }

        let encoding = hf_tokenizer
            .tokenizer
            .encode(from_utf8(data).unwrap(), false)
            .unwrap();
        let total_no_tokens: usize = lite_results.iter().map(|res| res.tokens.len()).sum();
        assert_eq!(encoding.len(), total_no_tokens);

        let all_tokens: Vec<u32> = lite_results
            .iter()
            .flat_map(|res| res.tokens.clone())
            .collect();
        assert_eq!(encoding.get_ids(), all_tokens);
    }

    #[test]
    fn none_superlinear_test() {
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "tests/error_data/2017_elections_in_India.txt";
        let data = fs::read(data_path).unwrap();
        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let splitter_config = SplitterLiteConfig::new_none(patterns, Some(512), None, false);

        let split_results = splitter_config.len_splits(data.as_slice());

        for res in split_results.iter() {
            println!("{:?}", res.split_string);
        }

        println!("Number of Split Results: {:?}", split_results.len());
        let total_len: usize = split_results.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_len);

        for res in split_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.split_string.len());
        }
    }

    #[test]
    fn max_len_splits() {
        let _data_raw = "\n\n\
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

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len: Some(16),
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
    }

    #[test]
    fn tree_visual_test() {
        let _data_raw = "\n\n\
        Returns and inequality, is so strong.\n\
        That it yields.\n\n\
        Another heuristic for.\n\n\
        \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len: None,
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_results_lite(None, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn tree_visual_hf_test() {
        let _data_raw = "\n\n\
        Returns and inequality, is so strong.\n\
        That it yields.\n\n\
        Another heuristic for.\n\n\
        \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, None, false).unwrap(),
        };

        let max_len = Some(8);

        let config = SplitterConfig::<HFTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&hf_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let splits = tree.get_node_splits(max_len, None);
        println!("Number of Splits: {:?}", splits.len());
        for split in splits.iter() {
            let split_str = from_utf8(&data[split.data_span.unwrap().range()]).unwrap();
            println!("{:?}", split_str);
            println!("{:?}", split.no_tokens());
        }
    }

    #[test]
    fn tree_visual_ws_test() {
        let _data_raw = "\n\n\
        Returns and inequality, is so strong.\n\
        That it yields.\n\n\
        Another heuristic for.\n\n\
        \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let ws_tokenizer = WSTokenizer { ascii: false };

        let config = SplitterConfig::<WSTokenizer> {
            data,
            searchers: &searchers,
            max_len: None,
            merge_level: None,

            tokenizer: Some(&ws_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_results_lite(None, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn tree_visual_max_len_test() {
        let _data_raw = "\n\n\
        Returns and inequality, is so strong.\n\
        That it yields.\n\n\
        Another heuristic for.\n\n\
        \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let max_len: Option<usize> = Some(16);
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_results_lite(None, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn tree_visual_ws_max_len_test() {
        let _data_raw = "\n\n\
        Returns and inequality, is so strong.\n\
        That it yields.\n\n\
        Another heuristic for.\n\n\
        \n\
        Outperform everyone else.\n\n"
            .as_bytes();

        let _data = "\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\
        Another heuristic for.\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n"
            .as_bytes();

        let data = _data_raw;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let ws_tokenizer = WSTokenizer { ascii: false };

        //let max_len: Option<usize> = Some();
        let max_len: Option<usize> = None;
        let config = SplitterConfig::<WSTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&ws_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_results_lite(None, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.split_string.len());
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn tree_visual_level_1_match_max_len_test() {
        let data = "\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\
        Another heuristic for.\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n"
            .as_bytes();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let max_len: Option<usize> = Some(56);
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_node_splits(max_len, None);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            let split_data_span = res.data_span.unwrap();
            let split_string =
                from_utf8(&data[split_data_span.start..split_data_span.end]).unwrap();
            println!("{:?}", split_string);
            println!("{:?}", split_string.len());
        }
    }

    #[test]
    fn tree_visual_ws_level_1_match_max_len_test() {
        let data = "\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\
        Another heuristic for.\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n"
            .as_bytes();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), ",".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let ws_tokenizer = WSTokenizer { ascii: false };

        let max_len: Option<usize> = Some(3);
        let config = SplitterConfig::<WSTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&ws_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_results_lite(None, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.split_string.len());
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn tree_visual_ws_no_match_max_len_test() {
        let file_path = "tests/test_data/en_long_sentence.txt";
        let data_string = fs::read_to_string(file_path).unwrap();
        let data = data_string.as_bytes();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string()],
        ];
        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let ws_tokenizer = WSTokenizer { ascii: false };

        let max_len: Option<usize> = Some(8);
        let config = SplitterConfig::<WSTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level: None,

            tokenizer: Some(&ws_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        //println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
        match term_tree(tree.clone(), data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

        let lite_results = tree.get_results_lite(None, None, data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.split_string.len());
            println!("{:?}", res.tokens.len());
        }
    }

    #[test]
    fn fast_none_tree_splits() {
        let data_path = "tests/test_data/superlinear.txt";
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();
        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let splitter_config = SplitterLiteConfig::new_none(patterns, Some(512), Some(4), false);
        let splits = splitter_config.len_splits(data);

        let chunk_lens: usize = splits.iter().map(|c| c.split_string.len()).sum();

        println!("{:?}", splits.len());
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.split_string.len());
        }
        assert_eq!(data.len(), chunk_lens);
    }

    #[test]
    fn nw_tree_splits_superlinear() {
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "tests/error_data/superlinear_loose_pattern.txt";
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();
        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let patterns_len = patterns.len();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let max_len = Some(256);
        //let merge_level = Some(2);
        let merge_level = None;
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            merge_level,
            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        let splits = tree.get_node_splits(max_len, merge_level);
        println!("Number of Splits: {:?}", splits.len());
        for res in splits.iter() {
            let split_data_span = res.data_span.unwrap();
            let split_string =
                from_utf8(&data[split_data_span.start..split_data_span.end]).unwrap();
            println!("{:?}", split_string);
            println!("{:?}", split_string.len());
            //println!("pattern: {:?}", res.pattern_id);
        }

        let total_len: usize = splits.iter().map(|res| res.no_tokens()).sum();
        assert_eq!(data.len(), total_len);

        match term_tree(tree, data) {
            Ok(tree) => {
                let mut file =
                    fs::File::create("output/debug/tree_len_mx_256_superlinear.txt").unwrap();
                file.write_all(format!("{}", tree).as_bytes()).unwrap();
                //println!("{}", tree)
            }
            Err(err) => println!("error: {}", err),
        }

        //let result_concated: String = splits.iter().map(|split| from_utf8(&data[split.data_span.unwrap().range()]).unwrap()).collect();
        // write to file output/debug/superlinear_splits_concated.txt
        // add date and time to file name
        //let mut file = fs::File::create("output/debug/superlinear_splits_concated.txt").unwrap();
        //if file doesn't exist, create it
        //file.write_all(result_concated.as_bytes()).unwrap();

        /*
        match term_tree(tree, data) {
            Ok(tree) => println!("{}", tree),
            Err(err) => println!("error: {}", err),
        }

         */
    }

    #[test]
    fn nw_tree_hf_splits_superlinear() {
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "tests/error_data/superlinear_loose_pattern.txt";
        //let data_path = "data/dev/History_of_baseball_in_the_United_States.txt";
        //let data_path = "tests/error_data/wiki_us_snippet_error.txt";
        //let data_path = "data/dev/Belle_(Beauty_and_the_Beast).txt";

        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let splitter_config = SplitterLiteConfig::new_hf(patterns, Some(512), None, true, None);

        let splits = splitter_config.hf_splits(data);

        println!("{:?}", splits.len());

        let total_len: usize = splits.iter().map(|c| c.split_string.len()).sum();
        assert_eq!(data.len(), total_len);

        let hf_tokenizer = init_tokenizer(None, Some(usize::MAX), false).unwrap();
        let hf_encoding = hf_tokenizer
            .encode(from_utf8(data).unwrap(), false)
            .unwrap();
        let total_tokens: usize = splits.iter().map(|res| res.tokens.len()).sum();

        assert_eq!(hf_encoding.len(), total_tokens);

        //write to file output/debug/hf_splits_concated.txt
        let result_concated: String = splits
            .iter()
            .map(|split| split.split_string.clone())
            .collect();
        let mut file = fs::File::create("output/debug/hf_splits_concated.txt").unwrap();
        file.write_all(result_concated.as_bytes()).unwrap();

        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.tokens.len());
            println!("{:?}", split.split_string.len());
            let hf_encoded = hf_tokenizer
                .encode(split.split_string.clone(), false)
                .unwrap();
            //assert!(!split.tokens.is_empty());
            assert!(split.tokens.len() <= 512);
            if split.tokens.len() != hf_encoded.len() {
                // tokens in hf_encoded but not in split.tokens
                let diff: Vec<u32> = hf_encoded
                    .get_ids()
                    .iter()
                    .filter(|&x| !split.tokens.contains(x))
                    .map(|&x| x)
                    .collect();
                println!("{:?}", diff.len());

                for token in diff.iter() {
                    println!("{:?}", hf_tokenizer.id_to_token(*token));
                }
                break;
            }
            assert_eq!(hf_encoded.len(), split.tokens.len());
            assert_eq!(hf_encoded.get_ids(), split.tokens);
        }
    }

    #[test]
    fn fast_hf_tree_splits() {
        let data_path = "tests/test_data/superlinear.txt";
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();
        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let splitter_config = SplitterLiteConfig::new_hf(patterns, Some(60), None, true, None);
        let splits = splitter_config.hf_splits(data);

        let chunk_lens: usize = splits.iter().map(|c| c.split_string.len()).sum();
        assert_eq!(chunk_lens, data.len());

        let hf_tokenizer = init_tokenizer(None, Some(usize::MAX), false).unwrap();
        let hf_encoding = hf_tokenizer
            .encode(from_utf8(data).unwrap(), false)
            .unwrap();

        let total_tokens: usize = splits.iter().map(|res| res.tokens.len()).sum();
        assert_eq!(hf_encoding.len(), total_tokens);

        println!("number of splits: {:?}", splits.len());
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.split_string.len());
            println!("{:?}", split.tokens.len());
            let hf_encoded = hf_tokenizer
                .encode(split.split_string.clone(), false)
                .unwrap();
            assert_eq!(hf_encoded.len(), split.tokens.len());
        }
    }

    #[test]
    fn fast_two_hf_tree_splits() {
        let data_path = "tests/test_data/superlinear.txt";
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let splitter_config = SplitterLiteConfig::new_hf(patterns, Some(512), None, true, None);
        let splits = splitter_config.hf_splits(data);

        let chunk_lens: usize = splits.iter().map(|c| c.split_string.len()).sum();
        assert_eq!(chunk_lens, data.len());

        let hf_tokenizer = init_tokenizer(None, Some(usize::MAX), false).unwrap();
        let hf_encoding = hf_tokenizer
            .encode(from_utf8(data).unwrap(), false)
            .unwrap();

        let total_tokens: usize = splits.iter().map(|res| res.tokens.len()).sum();
        assert_eq!(hf_encoding.len(), total_tokens);

        println!("number of splits: {:?}", splits.len());
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.split_string.len());
            println!("{:?}", split.tokens.len());
            let hf_encoded = hf_tokenizer
                .encode(split.split_string.clone(), false)
                .unwrap();
            assert_eq!(hf_encoded.len(), split.tokens.len());
        }

        println!("#### Sub Splits ####");
        let sub_patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let sub_splitter_config =
            SplitterLiteConfig::new_hf(sub_patterns, Some(512), Some(3), true, None);
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.split_string.len());
            let sub_splits = sub_splitter_config.hf_splits(split.split_string.as_bytes());
            for sub_split in sub_splits.iter() {
                println!("{:?}", sub_split.split_string);
                println!("{:?}", sub_split.split_string.len());
                println!("{:?}", sub_split.tokens.len());
                let hf_encoded = hf_tokenizer
                    .encode(sub_split.split_string.clone(), false)
                    .unwrap();
                assert_eq!(hf_encoded.len(), sub_split.tokens.len());
            }
            let total_len: usize = sub_splits.iter().map(|res| res.split_string.len()).sum();
            assert_eq!(split.split_string.len(), total_len);
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

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let patterns_len = patterns.len();

        //let patterns = vec!["\n\n".to_string()];
        //let patterns = vec!["\n\n".to_string(), "\n".to_string()];
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            searchers: &searchers,
            max_len: None,
            merge_level: None,

            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
    }

    #[test]
    fn pattern_split_superlinear_print() {
        //let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";
        let data_path = "tests/test_data/Belle_(Beauty_and_the_Beast).txt";

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![". ".to_string(), "!".to_string(), "?".to_string()],
        ];
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let (data_len, splits) = split_file(
            data_path, patterns, &searchers, 512, 2, true, true, true, true,
        );

        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data_len, total_len);
        println!("Total Leaves: {:?}", splits.len());
        println!("Total Length: {:?}", total_len);
    }

    #[test]
    fn chinese_hf_split_print() {
        let data_path = "tests/test_data/chinese_example01.txt";
        let data = fs::read(data_path).unwrap();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![
                ".".to_string(),
                ". ".to_string(),
                ",".to_string(),
                ", ".to_string(),
                "、".to_string(),
                "。".to_string(),
            ],
        ];
        let splitter = SplitterLiteConfig::new_hf(
            patterns,
            Some(512),
            None,
            true,
            Some("sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2".to_string()),
        );

        let splits = splitter.hf_splits(data.as_slice());

        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_len);
        println!("Total Leaves: {:?}", splits.len());
        println!("Total Length: {:?}", total_len);

        // get splitter tokenizer as HFTokenizer
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.tokens.len());
        }
    }

    #[test]
    fn chinese_ws_split_print() {
        let data_path = "tests/test_data/chinese_example01.txt";
        let orig_data_str = fs::read_to_string(data_path).unwrap();

        let normalizer = TextNormalizer::default();
        let data_str = normalizer.normalize(&orig_data_str).unwrap();
        let data = data_str.as_str();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![
                ".".to_string(),
                ". ".to_string(),
                ",".to_string(),
                ", ".to_string(),
                "、".to_string(),
                "。".to_string(),
            ],
        ];
        let splitter = SplitterLiteConfig::new_ws(patterns, Some(512), None, true, false);

        let splits = splitter.ws_splits(data.as_bytes());

        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();

        assert_eq!(data.len(), total_len);
        println!("Total Leaves: {:?}", splits.len());
        println!("Total Length: {:?}", total_len);

        // get splitter tokenizer as HFTokenizer
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.tokens.len());
        }
        let denormalized_split_strings: Vec<String> = splits
            .iter()
            .map(|res| normalizer.denormalize(&res.split_string).unwrap())
            .collect();
        for split in denormalized_split_strings.iter() {
            println!("{:?}", split);
        }

        let denormalized_len: usize = denormalized_split_strings.iter().map(|res| res.len()).sum();
        assert_eq!(orig_data_str.len(), denormalized_len);
    }

    #[test]
    fn chinese_none_split_print() {
        let data_path = "tests/test_data/chinese_example01.txt";
        let data_str = fs::read_to_string(data_path).unwrap();
        let data = data_str.as_str();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![
                ".".to_string(),
                ". ".to_string(),
                ",".to_string(),
                ", ".to_string(),
                "、".to_string(),
                "。".to_string(),
            ],
        ];
        let splitter = SplitterLiteConfig::new_none(patterns, Some(512), None, false);

        let splits = splitter.len_splits(data.as_bytes());

        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();

        assert_eq!(data.len(), total_len);
        println!("Total Leaves: {:?}", splits.len());
        println!("Total Length: {:?}", total_len);

        // get splitter tokenizer as HFTokenizer
        for split in splits.iter() {
            println!("{:?}", split.split_string);
            println!("{:?}", split.tokens.len());
        }

        assert_eq!(data.len(), total_len);
    }

    #[test]
    fn hf_nq_dataset_test() -> tokenizers::Result<()> {
        let mut rng = thread_rng();

        tokenizers::utils::parallelism::set_parallelism(true);
        let dev_files = list_text_files("data/dev/")?;
        let train_files = list_text_files("data/train/")?;
        let files: Vec<_> = dev_files.iter().chain(train_files.iter()).collect();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![". ".to_string(), "!".to_string(), "?".to_string()],
        ];
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let rnd_files: Vec<_> = files.choose_multiple(&mut rng, 3000).collect();

        rnd_files.par_iter().for_each(|file| {
            for merge_level in 1..3 {
                let (data_len, results) = split_file(
                    file,
                    patterns.clone(),
                    &searchers,
                    512,
                    merge_level,
                    false,
                    false,
                    true,
                    true,
                );
                let total_len: usize = results.iter().map(|res| res.split_string.len()).sum();
                assert_eq!(data_len, total_len, "Failed File: {}", file);
                println!("File: {} \n Total Length: {}", file, total_len);
                println!("Number of Splits: {:?}", results.len());
            }
        });

        Ok(())
    }

    #[test]
    fn test_domcura() -> tokenizers::Result<()> {
        let data_path = "tests/test_data/vertragsgrundlagen_efh_wohn.txt";
        let binding = fs::read(data_path).unwrap();
        let data = binding.as_slice();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let splitter_config = SplitterLiteConfig::new_hf(
            patterns,
            Some(512),
            None,
            true,
            Some("sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2".to_string()),
        );
        let splits = splitter_config.hf_splits(data);
        println!("{:?}", splits.len());
        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_len);

        for res in splits.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }

        // write splits to output/debug/vertragsgrundlagen_efh_wohn_splits.txt
        let mut file =
            fs::File::create("output/debug/vertragsgrundlagen_efh_wohn_splits.txt").unwrap();
        for res in splits.iter() {
            file.write_all(res.split_string.as_bytes()).unwrap();
        }

        Ok(())
    }
    #[test]
    fn test_turkish() -> tokenizers::Result<()> {
        let data_path = "tests/test_data/telekomturkishsample.txt";
        let binding = fs::read(data_path).unwrap();
        let data = binding.as_slice();

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];

        let splitter_config = SplitterLiteConfig::new_hf(
            patterns,
            Some(512),
            None,
            true,
            Some("sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2".to_string()),
        );
        let splits = splitter_config.hf_splits(data);
        println!("{:?}", splits.len());
        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data.len(), total_len);

        for res in splits.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }

        // write splits to output/debug/vertragsgrundlagen_efh_wohn_splits.txt
        let mut file =
            fs::File::create("output/debug/vertragsgrundlagen_efh_wohn_splits.txt").unwrap();
        for res in splits.iter() {
            file.write_all(res.split_string.as_bytes()).unwrap();
        }

        Ok(())
    }
}
