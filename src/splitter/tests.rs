#[cfg(test)]
mod tests {
    use std::str::from_utf8;
    use std::{fs, io};

    use aho_corasick::Span;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    use crate::encodings::{NoneTokenizer, Tokenize};
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
    ) -> (usize, Vec<Span>) {
        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let span = Span {
            start: 0,
            end: data.len(),
        };
        let config = SplitterConfig::<NoneTokenizer> {
            data,
            patterns,
            searchers,
            max_len: Some(max_len),
            tokenizer: None,
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();

        let leaf_level = tree.leaf_level().unwrap_or(0);
        println!("Leaf Level: {:?}", leaf_level);
        let splits = tree.merge_splits(leaf_level, Some(max_len));
        for span in splits.iter() {
            assert!(
                span.len() <= max_len,
                "Failed File: {} \n Span {:?} \n {}",
                span.len(),
                span,
                from_utf8(&data[span.start..span.end]).unwrap()
            );
        }

        if _print {
            for span in splits.iter() {
                println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
                println!("Length: {:?}", span.len());
            }
        }
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

        println!("error data {:?}", from_utf8(&data[135..161]).unwrap());

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let span = Span {
            start: 0,
            end: data.len(),
        };
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();
        let ws_tokenizer = WSTokenizer {};

        let config = SplitterConfig::<WSTokenizer> {
            data,
            patterns: &patterns,
            searchers: &searchers,
            max_len: Some(20),
            tokenizer: Some(&ws_tokenizer),
        };
        let splitter = Splitter::new(&config, span, 0, span, None, None, None);
        let tree = splitter.split();
        println!("{}", tree.to_string(data, true));

        let leaf_level_opt = tree.leaf_level();
        assert_eq!(leaf_level_opt, Some(1));

        let splits_encoding = tree.merge_splits_encoding(leaf_level_opt.unwrap(), Some(20));

        let splits: Vec<_> = splits_encoding
            .iter()
            .map(|(_, data_span)| data_span.clone())
            .collect();

        let total_len: usize = splits.iter().map(|span| span.len()).sum();
        //assert_eq!(data.len(), total_len);
        println!("Number of Splits: {:?}", splits.len());

        let ws_tokenizer = WSTokenizer {};
        for span in splits.iter() {
            let split_str = from_utf8(&data[*span]).unwrap();
            println!("{:?}", split_str);
            let encoded = ws_tokenizer.encode(split_str).unwrap();
            println!("Tokens len: {:?}", encoded.len());
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

        let binding = fs::read_to_string(data_path).unwrap();
        let data = binding.as_bytes();

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec!["."]];

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
        let reconsted = tree.reconstruct(data);
        assert_eq!(from_utf8(&data).unwrap(), reconsted);
    }

    #[test]
    fn pattern_split_superlinear_print() {
        let data_path = "tests/test_data/superlinear.txt";
        //let data_path = "data/train/List_of_Game_of_Thrones_characters.txt";

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();

        let (data_len, splits) = split_file(data_path, &patterns, &searchers, 256, true);

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
        let files = list_text_files("data/dev/")?;

        let patterns = vec![vec!["\n\n"], vec!["\n"], vec![".", "!", "?"]];
        let searchers: Vec<_> = patterns.iter().map(|p| PatternSearcher::new(p)).collect();

        files.choose_multiple(&mut rng, 400).for_each(|file| {
            let (data_len, splits) = split_file(file, &patterns, &searchers, 512, false);
            let total_len: usize = splits.iter().map(|span| span.len()).sum();
            assert_eq!(data_len, total_len, "Failed File: {}", file);
            println!("File: {} \n Total Length: {}", file, total_len);
            println!("Number of Splits: {:?}", splits.len());
        });

        Ok(())
    }
}
