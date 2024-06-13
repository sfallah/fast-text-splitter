#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::str::from_utf8;
    use std::{fs, io};
    use rayon::prelude::*;

    use aho_corasick::Span;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    use crate::encodings::{NoneTokenizer, Tokenize};
    use crate::hf_tokenizer::{init_tokenizer, HFTokenizer};
    use crate::pattern_search::pattern_searcher::PatternSearcher;
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
        patterns: &Vec<Vec<&str>>,
        searchers: &Vec<PatternSearcher>,
        max_len: usize,
        _print: bool,
        check_splits: bool
    ) -> (usize, Vec<Span>) {
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, Some(data.len()), false).unwrap(),
        };

        let max_len = Some(max_len);
        //let config = SplitterConfig::<NoneTokenizer> {
        let config = SplitterConfig::<HFTokenizer> {
            data,
            patterns,
            searchers,
            max_len,
            //tokenizer: None,
            tokenizer: Some(&hf_tokenizer),
        };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();

        let leaf_level = tree.leaf_level().unwrap_or(0);

        println!("Leaf Level: {:?}", leaf_level);
        let splits_encoding = tree.merge_splits_encoding(leaf_level, max_len);

        let splits: Vec<_> = splits_encoding
            .iter()
            .map(|(_, data_span)| data_span.clone())
            .collect();

        if _print {
            for span in splits.iter() {
                println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
                println!("Length: {:?}", span.len());
            }
        }

        let splits_encoding = tree.merge_splits_encoding(leaf_level, max_len);

        let splits: Vec<_> = splits_encoding
            .iter()
            .map(|(_, data_span)| data_span.clone())
            .collect();

        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        assert_eq!(data.len(), total_len);
        println!("Number of Splits: {:?}", splits.len());

        let split_results =
            tree.merge_encoding_result(leaf_level, max_len, from_utf8(data).unwrap());

        println!("Number of Split Results: {:?}", split_results.len());
        assert_eq!(split_results.len(), splits.len());

        if check_splits {

            for (span, result) in splits_encoding.iter().zip(split_results.iter()) {
                println!("Span: {:?}", span);
                let split_str = from_utf8(&data[span.1.range()]).unwrap();
                println!("{:?}", split_str);

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
        }


        let total_tokens: usize = split_results
            .iter()
            .map(|res| res.results.as_ref().unwrap().ids.len())
            .sum();
        println!("Total Tokens: {:?}", total_tokens);
        assert_eq!(total_tokens, hf_tokenizer.encode(from_utf8(data).unwrap()).unwrap().len());

        (data.len(), splits)
    }

    #[test]
    fn single_level_split() {
        let data = "Hello, you all.\n How are you.\n".as_bytes();
        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len: None,
            tokenizer: None,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        println!("{}", tree.to_string(data, true));
        let reconsted = tree.reconstruct(data);
        assert_eq!(from_utf8(data).unwrap(), reconsted);
    }

    #[test]
    fn multi_level_split() {
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

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len: None,
            tokenizer: None,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
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

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len: Some(16),
            tokenizer: None,
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

        let splits = tree.merge_splits(tree.leaf_level().unwrap(), Some(16));
        for span in splits.iter() {
            println!("{:?}", from_utf8(&data[*span]).unwrap());
        }
        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        assert_eq!(data.len(), total_len);
    }

    #[test]
    fn with_encoding_superlinear_test() {
        let data_path = "tests/test_data/superlinear.txt";
        let data_path = "tests/error_data/2017_elections_in_India.txt";

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

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let ws_tokenizer = WSTokenizer {};

        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, None, false).unwrap(),
        };

        let max_len = Some(128);
        let config = SplitterConfig::<HFTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len,
            //tokenizer: Some(&ws_tokenizer),
            tokenizer: Some(&hf_tokenizer),
        };
        let splitter = Splitter::new(&config, span, 0, span, Some(true), None, None);
        let tree = splitter.split();

        //write to file
        //let mut file = fs::File::create("output/test_data/superlinear_tree.txt").unwrap();
        //if file doesn't exist, create it

        //file.write_all(tree.to_string(data, true).as_bytes()).unwrap();
        //println!("{}", tree.to_string(data, true));

        let leaf_level_opt = tree.leaf_level();
        //assert_eq!(leaf_level_opt, Some(1));

        let splits_encoding = tree.merge_splits_encoding(leaf_level_opt.unwrap(), max_len);

        let splits: Vec<_> = splits_encoding
            .iter()
            .map(|(_, data_span)| data_span.clone())
            .collect();

        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        assert_eq!(data.len(), total_len);
        println!("Number of Splits: {:?}", splits.len());

        let split_results =
            tree.merge_encoding_result(leaf_level_opt.unwrap(), max_len, from_utf8(data).unwrap());

        println!("Number of Split Results: {:?}", split_results.len());
        assert_eq!(split_results.len(), splits.len());

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

        let total_tokens: usize = split_results
            .iter()
            .map(|res| res.results.as_ref().unwrap().ids.len())
            .sum();
        println!("Total Tokens: {:?}", total_tokens);
        assert_eq!(total_tokens, hf_tokenizer.encode(from_utf8(data).unwrap()).unwrap().len());
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

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        //let patterns = vec!["\n\n".to_string()];
        //let patterns = vec!["\n\n".to_string(), "\n".to_string()];
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len: None,
            tokenizer: None,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));
        assert_eq!(from_utf8(data).unwrap(), tree.reconstruct(data));
    }

    #[test]
    fn pattern_split_superlinear_test() {
        let data_path = "tests/test_data/superlinear.txt";
        let data_path = "tests/error_data/2017_elections_in_India.txt";

        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec!["."]];

        let span = Span {
            start: 0,
            end: data.len(),
        };

        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let max_len = Some(40);
        let hf_tokenizer = HFTokenizer {
            tokenizer: init_tokenizer(None, None, false).unwrap(),
        };
        //let config = SplitterConfig::<WSTokenizer> {
        let config = SplitterConfig::<HFTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len,
            //tokenizer: None,
            //tokenizer: Some(&WSTokenizer {}),
            tokenizer: Some(&hf_tokenizer),
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        //println!("{}", tree.to_string(data, true));
        //let splits = tree.merge_splits(tree.leaf_level().unwrap(), max_len);
        let splits_enc = tree.merge_splits_encoding(tree.leaf_level().unwrap(), max_len);
        let total_len_enc: usize = splits_enc.iter().map(|span| span.1.len()).sum();
        let splits: Vec<_> = splits_enc.iter().map(|(_, span)| span.clone()).collect();
        println!("Number of Splits: {:?}", splits.len());
        let total_len: usize = splits.iter().map(|sp| sp.len()).sum();

        let splits_concat: String = splits
            .iter()
            .map(|kv| from_utf8(&data[kv.range()]).unwrap())
            .collect();
        // write to file
        let mut file = fs::File::create("output/debug/merged_2017_elections_in_India.txt").unwrap();
        // directory must exist
        file.write_all(splits_concat.as_bytes()).unwrap();
        assert_eq!(data.len(), total_len_enc);
        println!("{:?}", splits_concat);
        assert_eq!(data.len(), total_len);
        let reconsted = tree.reconstruct(data);
        assert_eq!(from_utf8(&data).unwrap(), reconsted);
    }

    #[test]
    fn pattern_split_superlinear_print() {
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();

        let (data_len, splits) = split_file(data_path, &patterns, &searchers, 256, true, true);

        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        assert_eq!(data_len, total_len);
        println!("Total Leaves: {:?}", splits.len());
        println!("Total Length: {:?}", total_len);
    }

    #[test]
    #[cfg(feature = "tokenizers")]
    fn hf_nq_dataset_test() -> tokenizers::Result<()> {
        let mut rng = thread_rng();

        tokenizers::utils::parallelism::set_parallelism(true);
        //let files = list_text_files("data/dev/")?;
        let files = list_text_files("data/train/")?;

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();

        let rnd_files: Vec<_> = files.choose_multiple(&mut rng, 2000).collect();

        rnd_files.par_iter().for_each(|file| {
            let (data_len, splits) = split_file(file, &patterns, &searchers, 512, false, false);
            let total_len: usize = splits.iter().map(|span| span.len()).sum();
            assert_eq!(data_len, total_len, "Failed File: {}", file);
            println!("File: {} \n Total Length: {}", file, total_len);
            println!("Number of Splits: {:?}", splits.len());
        });

        Ok(())
    }
}
