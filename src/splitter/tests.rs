#[cfg(test)]
mod tests {
    use std::str::from_utf8;
    use std::{fs, io};

    use aho_corasick::Span;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use rayon::prelude::*;

    use crate::encodings::{NoneTokenizer, Tokenize};
    use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
    use crate::pattern_search::pattern_searcher::PatternSearcher;
    use crate::splitter::split_node::utils::SplitResultLite;
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
        _print: bool,
        check_splits: bool,
        parallel: bool,
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

        let leaf_level = tree.leaf_level().unwrap_or(0);

        println!("Leaf Level: {:?}", leaf_level);

        let split_results = tree.get_results(max_len, from_utf8(data).unwrap());
        println!("Number of Split Results: {:?}", split_results.len());

        if _print {
            for res in split_results.iter() {
                println!("{:?}", res.split_strings);
                println!("Data Len: {:?}", res.split_strings.len());
                println!("Tokens Len: {:?}", res.results.as_ref().unwrap().ids.len());
            }
        }

        let total_text_len: usize = split_results
            .iter()
            .map(|res| res.split_strings.len())
            .sum();
        assert_eq!(data.len(), total_text_len);

        //assert_eq!(split_results.len(), splits.len());
        let total_res_len: usize = split_results
            .iter()
            .map(|res| res.split_strings.len())
            .sum();
        assert_eq!(data.len(), total_res_len);

        if check_splits {
            for res in split_results.iter() {
                println!("{:?}", res.split_strings);

                let hf_encoded = hf_tokenizer.encode(&res.split_strings).unwrap();
                println!("HF Tokens len: {:?}", hf_encoded.len());

                assert_eq!(
                    hf_encoded.len(),
                    res.clone().results.unwrap().ids.len(),
                    "Failed Split: {:?}\n Tokens-Result: {:?}\n Tokens Spans: {:?}",
                    res.split_strings,
                    res,
                    origin_dta_span
                );
            }
        }

        let total_tokens: usize = split_results
            .iter()
            .map(|res| res.results.as_ref().unwrap().ids.len())
            .sum();
        println!("Total Tokens: {:?}", total_tokens);
        let encodings = hf_tokenizer.encode(from_utf8(data).unwrap()).unwrap();
        assert_eq!(total_tokens, encodings.len());

        let lite_results = tree.get_results_lite(max_len, data);
        let total_lite_tokens: usize = lite_results.iter().map(|res| res.tokens.len()).sum();
        println!("Total Lite Tokens: {:?}", total_lite_tokens);

        assert_eq!(encodings.len(), total_lite_tokens);
        (data.len(), lite_results)
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
            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        println!("{}", tree.to_string(data, true));
        let reconsted = tree.reconstruct(data);
        assert_eq!(from_utf8(data).unwrap(), reconsted);

        let lite_results = tree.get_results_lite(Some(20), data);
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
            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));

        let lite_results = tree.get_results_lite(Some(32), data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("text: {:?}", res.split_string);
            println!("len: {:?}", res.split_string.len());
        }
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
            tokenizer: Some(&ws_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));

        let lite_results = tree.get_results_lite(max_len, data);
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

        let hf_tokenizer = HFTokenizer {
            tokenizer,
        };

        let max_len = Some(16);

        let config = SplitterConfig::<HFTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            tokenizer: Some(&hf_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        let lite_results = tree.get_results_lite(Some(16), data);
        println!("Number of Lite Results: {:?}", lite_results.len());
        for res in lite_results.iter() {
            println!("{:?}", res.split_string);
            println!("{:?}", res.tokens.len());
        }

        let encoding = hf_tokenizer.tokenizer.encode(from_utf8(data).unwrap(),false).unwrap();
        let total_no_tokens: usize = lite_results.iter().map(|res| res.tokens.len()).sum();
        assert_eq!(encoding.len(), total_no_tokens);

        let all_tokens: Vec<u32> = lite_results.iter().flat_map(|res| res.tokens.clone()).collect();
        assert_eq!(encoding.get_ids(), all_tokens);
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
            tokenizer: None,
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));

        let tree_leaves = tree.all_leaves();
        for span in tree_leaves.iter() {
            println!("{:?}", from_utf8(&data[*span]).unwrap());
        }
        let total_len: usize = tree_leaves.iter().map(|span| span.len()).sum();
        let leaves_concatenated: String = tree_leaves
            .iter()
            .map(|span| from_utf8(&data[span.start..span.end]).unwrap())
            .collect();
        println!("{:?}", leaves_concatenated);
        assert_eq!(data.len(), total_len);

        //FIXME: add the new relevant test for max_len none tokenizer

        /*
        let splits = tree.merge_splits(tree.leaf_level().unwrap(), Some(16));
        for span in splits.iter() {
            println!("{:?}", from_utf8(&data[*span]).unwrap());
        }
        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        assert_eq!(data.len(), total_len);

         */
    }

    #[test]
    fn with_encoding_superlinear_test() {
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "tests/error_data/2017_elections_in_India.txt";

        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let _data = "\n\n\
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
        //let data = _data;

        let _data = "\n\
        In fact, the correlation. Between superlinear.\n\
        Returns and inequality is so strong that it yields.\n\
        Another heuristic for.\n\
        Finding work of this type.\n\
        Look for fields where.\n\
        A few big winners. \n\
        Outperform everyone else.\n"
            .as_bytes();

        //let data = _data;

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
        //let ws_tokenizer = WSTokenizer {};

        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, None, false).unwrap(),
        };

        let max_len = Some(512);
        let config = SplitterConfig::<HFTokenizer> {
            data,
            searchers: &searchers,
            max_len,
            //tokenizer: Some(&ws_tokenizer),
            tokenizer: Some(&hf_tokenizer),
            patterns_len,
        };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();

        //write to file
        //let mut file = fs::File::create("output/test_data/superlinear_tree.txt").unwrap();
        //if file doesn't exist, create it

        //file.write_all(tree.to_string(data, true).as_bytes()).unwrap();
        //println!("{}", tree.to_string(data, true));

        //assert_eq!(leaf_level_opt, Some(1));

        let split_results = tree.get_results(max_len, from_utf8(data).unwrap());

        println!("Number of Split Results: {:?}", split_results.len());
        //assert_eq!(split_results.len(), splits.len());

        /*
        let ws_tokenizer = WSTokenizer {};
        for (span, result) in splits_encoding.iter().zip(split_results.iter()) {
            println!("Span: {:?}", span);
            let split_str = from_utf8(&data[span.1.range()]).unwrap();
            println!("{:?}", split_str);
            let ws_encoded = ws_tokenizer.encode(split_str).unwrap();
            println!("WS Tokens len: {:?}", ws_encoded.len());

            let hf_encoded = hf_tokenizer.encode(split_str).unwrap();
            println!("HF Tokens len: {:?}", hf_encoded.len());

            println!("{:?}", result);
            assert_eq!(
                hf_encoded.len(),
                result.clone().results.unwrap().ids.len(),
                "Failed Split: {:?}\n Tokens-Result: {:?}\n Tokens Spans: {:?}",
                split_str,
                result,
                span
            );
        }

         */

        let ws_tokenizer = WSTokenizer { ascii: false };
        for res in split_results.iter() {
            let split_str = res.split_strings.as_str();
            println!("{:?}", split_str);

            let ws_encoded = ws_tokenizer.encode(split_str).unwrap();
            println!("WS Tokens len: {:?}", ws_encoded.len());

            let hf_encoded = hf_tokenizer.encode(split_str).unwrap();
            println!("HF Tokens len: {:?}", hf_encoded.len());

            assert_eq!(
                hf_encoded.len(),
                res.clone().results.unwrap().ids.len(),
                "Failed Split: {:?}\n Tokens-Result: {:?}\n Tokens Spans: {:?}",
                split_str,
                res,
                span
            );
        }

        let total_tokens: usize = split_results
            .iter()
            .map(|res| res.results.as_ref().unwrap().ids.len())
            .sum();
        println!("Total Tokens: {:?}", total_tokens);
        assert_eq!(
            total_tokens,
            hf_tokenizer.encode(from_utf8(data).unwrap()).unwrap().len()
        );
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
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let (data_len, splits) = split_file(data_path, patterns, &searchers, 40, true, true, false);

        let total_len: usize = splits.iter().map(|res| res.split_string.len()).sum();
        assert_eq!(data_len, total_len);
        println!("Total Leaves: {:?}", splits.len());
        println!("Total Length: {:?}", total_len);
    }

    #[test]
    fn hf_nq_dataset_test() -> tokenizers::Result<()> {
        let mut rng = thread_rng();

        tokenizers::utils::parallelism::set_parallelism(true);
        //let files = list_text_files("data/dev/")?;
        let files = list_text_files("data/train/")?;

        let patterns = vec![
            vec!["\n\n".to_string()],
            vec!["\n".to_string()],
            vec![".".to_string(), "!".to_string(), "?".to_string()],
        ];
        let searchers: Vec<_> = patterns
            .iter()
            .map(|p| PatternSearcher::new(p.clone()))
            .collect();

        let rnd_files: Vec<_> = files.choose_multiple(&mut rng, 600).collect();

        rnd_files.par_iter().for_each(|file| {
            let (data_len, results) =
                split_file(file, patterns.clone(), &searchers, 512, false, false, true);
            let total_len: usize = results.iter().map(|res| res.split_string.len()).sum();
            assert_eq!(data_len, total_len, "Failed File: {}", file);
            println!("File: {} \n Total Length: {}", file, total_len);
            println!("Number of Splits: {:?}", results.len());
        });

        Ok(())
    }
}
