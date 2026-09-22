#[cfg(test)]
mod tests {
    use rayon::prelude::*;
    use tokenizers::Tokenizer;

    use crate::config::SplitterLiteConfig;
    use crate::encodings::{NoneTokenizer, Tokenize};
    use crate::hf_tokenizer::HFTokenizer;
    use crate::normalizer::TextNormalizer;
    use crate::pattern_search::code_patterns::Language;
    use sentence_splitter::Segmenter;
    use crate::splitter::split_node::utils::SplitResultLite;
    use crate::splitter::split_node::visualization::term_tree;
    use crate::test_support::*;
    use crate::ws_tokenizer::WSTokenizer;

    // ---------------------------------------------------------------------------------
    // Tree construction through the low-level `Splitter` API
    // ---------------------------------------------------------------------------------

    /// Build a tree and check what every tree must satisfy regardless of tokenizer: it
    /// reconstructs its input, renders, respects `max_len` per node, and flattens to
    /// splits that tile the input.
    fn check_tree<T: Tokenize + Sync>(
        data: &[u8],
        patterns: &[Vec<String>],
        tokenizer: Option<&T>,
        max_len: Option<usize>,
    ) -> Vec<SplitResultLite> {
        let searchers = searchers(patterns);
        let tree = split_tree(data, &searchers, tokenizer, max_len, None);

        assert_eq!(tree.reconstruct(data).as_bytes(), data);
        term_tree(&tree, data).expect("tree renders");

        if let Some(max_len) = max_len {
            for split in tree.get_node_splits(Some(max_len), None) {
                assert!(
                    split.no_tokens() <= max_len,
                    "node exceeds max_len {max_len}: {split:?}"
                );
            }
        }

        let splits = tree.get_results_lite(max_len, None, data);
        assert_tiles(data, &splits);
        splits
    }

    /// (input, patterns, max_len) combinations that exercise every branch of `Splitter::split`:
    /// leading/trailing/repeated separators, a first level that never matches, no match at
    /// all, a limit below the smallest natural unit, and no limit.
    fn tree_cases() -> Vec<(Vec<u8>, Vec<Vec<String>>, Option<usize>)> {
        let punctuation = patterns(&[&["\n\n"], &["\n"], &[".", ","]]);
        vec![
            (SHORT.into(), punctuation.clone(), None),
            (SHORT.into(), punctuation.clone(), Some(8)),
            (SHORT.into(), punctuation, Some(16)),
            (PARAGRAPHS.into(), paragraph_patterns(), Some(4)),
            (PARAGRAPHS.into(), paragraph_patterns(), Some(16)),
            (PARAGRAPHS.into(), paragraph_patterns(), Some(32)),
            (LINES.into(), paragraph_patterns(), Some(3)),
            (LINES.into(), paragraph_patterns(), Some(56)),
            (LINES.into(), sentence_first_patterns("<SENT>"), None),
            (
                b"this is a sentence".to_vec(),
                paragraph_patterns(),
                Some(512),
            ),
            (
                read("tests/test_data/en_long_sentence.txt"),
                patterns(&[&["\n\n"], &["\n"], &["."]]),
                Some(8),
            ),
            (
                read("tests/test_data/superlinear.txt"),
                paragraph_patterns(),
                Some(256),
            ),
        ]
    }

    #[test]
    fn tree_without_tokenizer() {
        for (data, patterns, max_len) in tree_cases() {
            check_tree::<NoneTokenizer>(&data, &patterns, None, max_len);
        }
    }

    #[test]
    fn tree_with_whitespace_tokenizer() {
        let ws = WSTokenizer { ascii: false };
        for (data, patterns, max_len) in tree_cases() {
            check_tree(&data, &patterns, Some(&ws), max_len);
        }
    }

    #[test]
    fn tree_with_hf_tokenizer() {
        let hf = hf_tokenizer(DEFAULT_MODEL);
        for (data, patterns, max_len) in tree_cases() {
            let splits = check_tree(&data, &patterns, Some(&hf), max_len);
            assert_token_partition(&hf.tokenizer, &data, &splits, max_len);
        }
    }

    #[test]
    fn splits_merge_up_to_max_len() {
        let data = b"Hello, you all.\n How are you.\n";
        let splitter = SplitterLiteConfig::new_none(paragraph_patterns(), Some(20), None, false);
        let splits = splitter.len_splits(data);
        assert_eq!(texts(&splits), ["Hello, you all.\n", " How are you.\n"]);
    }

    // ---------------------------------------------------------------------------------
    // Whole files through the public `SplitterLiteConfig` API
    // ---------------------------------------------------------------------------------

    #[test]
    fn none_tokenizer_splits_tile_file() {
        let data = read("tests/test_data/superlinear.txt");
        for merge_level in [None, Some(4)] {
            let splitter =
                SplitterLiteConfig::new_none(paragraph_patterns(), Some(512), merge_level, false);
            assert_tiles(&data, &splitter.len_splits(&data));
        }
    }

    struct HfFileCase {
        path: &'static str,
        patterns: Vec<Vec<String>>,
        max_tokens: usize,
        model: &'static str,
        /// The splits' tokens, concatenated, are the whole-document encoding.
        partitions: bool,
        /// Re-encoding a split's text reproduces its stored tokens.
        ///
        /// Both hold for WordPiece unless `max_tokens` forces a cut inside a word. Neither
        /// holds for sentencepiece models: they emit a standalone `▁` wherever a piece is
        /// encoded without its left context, so tokens depend on where a piece starts.
        reencodes: bool,
    }

    fn hf_file_cases() -> Vec<HfFileCase> {
        let cjk = patterns(&[&["\n\n"], &["\n"], &[".", ". ", ",", ", ", "、", "。"]]);
        vec![
            HfFileCase {
                path: "tests/test_data/superlinear.txt",
                patterns: paragraph_patterns(),
                max_tokens: 512,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: true,
            },
            HfFileCase {
                path: "tests/test_data/superlinear.txt",
                patterns: sentence_first_patterns("<SENT>"),
                max_tokens: 60,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: true,
            },
            HfFileCase {
                path: "tests/test_data/en_long_paragraphs.txt",
                patterns: paragraph_patterns(),
                max_tokens: 128,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: true,
            },
            HfFileCase {
                path: "tests/test_data/paper_arxiv_org__2108.07258v3.txt",
                patterns: sentence_first_patterns("<SENT>"),
                max_tokens: 510,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: true,
            },
            HfFileCase {
                path: "tests/test_data/paper_arxiv_org__2108.07258v3.txt",
                patterns: patterns(&[&["\n\n"], &["\n"], &[".", "!", "?", ". "]]),
                max_tokens: 510,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: true,
            },
            // Keyword separators with word tokens above the last level.
            HfFileCase {
                path: "tests/test_data_code/text_splitters.py",
                patterns: Language::PYTHON.get_separators(),
                max_tokens: 128,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: true,
            },
            // Text without spaces: every chunk is a cut inside one huge "word".
            HfFileCase {
                path: "tests/test_data/newsroom_no_space.txt",
                patterns: paragraph_patterns(),
                max_tokens: 64,
                model: DEFAULT_MODEL,
                partitions: true,
                reencodes: false,
            },
            HfFileCase {
                path: "tests/test_data/chinese_example01.txt",
                patterns: cjk,
                max_tokens: 512,
                model: MULTILINGUAL_MODEL,
                partitions: false,
                reencodes: false,
            },
            HfFileCase {
                path: "tests/test_data/vertragsgrundlagen_efh_wohn.txt",
                patterns: paragraph_patterns(),
                max_tokens: 512,
                model: MULTILINGUAL_MODEL,
                partitions: false,
                reencodes: false,
            },
            HfFileCase {
                path: "tests/test_data/telekomturkishsample.txt",
                patterns: paragraph_patterns(),
                max_tokens: 512,
                model: MULTILINGUAL_MODEL,
                partitions: false,
                reencodes: false,
            },
        ]
    }

    #[test]
    fn hf_splits_of_files_keep_their_tokens() {
        for case in hf_file_cases() {
            println!("{} with {:?}", case.path, case.patterns);
            let data = read(case.path);
            let splitter = hf_lite(case.patterns, Some(case.max_tokens), None, true, case.model);
            let splits = splitter.hf_splits(&data);
            let tokenizer = &splitter.tokenizer.tokenizer;

            assert_tiles(&data, &splits);
            for split in &splits {
                assert!(
                    split.tokens.len() <= case.max_tokens,
                    "{:?}",
                    split.split_string
                );
            }
            if case.partitions {
                assert_token_partition(tokenizer, &data, &splits, Some(case.max_tokens));
            }
            if case.reencodes {
                assert_splits_reencode(tokenizer, &splits);
            }
        }
    }

    /// No separator matches anywhere, so the splitter must hard-cut by token count, backing
    /// off to word boundaries. (One unbroken run of characters would not do: WordPiece maps
    /// any word longer than 100 characters to a single `[UNK]`.)
    #[test]
    fn hf_splits_text_without_any_separator() {
        use rand::distr::{Alphanumeric, SampleString};
        use rand::{rngs::StdRng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(7);
        let words: Vec<String> = (0..400)
            .map(|_| Alphanumeric.sample_string(&mut rng, 5))
            .collect();
        let data = words.join(" ");

        let splitter = hf_lite(paragraph_patterns(), Some(128), None, true, DEFAULT_MODEL);
        let tokenizer = &splitter.tokenizer.tokenizer;
        assert!(
            tokenizer.encode(data.as_str(), false).unwrap().len() > 128,
            "input must not fit in one chunk"
        );

        let splits = splitter.hf_splits(data.as_bytes());
        assert!(splits.len() > 1);
        assert_tiles(data.as_bytes(), &splits);
        assert_token_partition(tokenizer, data.as_bytes(), &splits, Some(128));
        assert_splits_reencode(tokenizer, &splits);
    }

    #[test]
    fn new_hf_clamps_merge_level_to_pattern_count() {
        let splitter =
            SplitterLiteConfig::new_hf(paragraph_patterns(), Some(512), Some(99), true, None);
        assert_eq!(splitter.merge_level, Some(3));
        assert_eq!(splitter.patterns_len, 3);
    }

    /// Re-splitting a chunk with `merge_level == patterns.len()` yields its finest pieces, which
    /// must still tile the chunk and partition its tokens.
    fn assert_resplit_is_lossless(
        tokenizer: &Tokenizer,
        sub_splitter: &SplitterLiteConfig<HFTokenizer>,
        splits: &[SplitResultLite],
    ) {
        splits.par_iter().for_each(|split| {
            let data = split.split_string.as_bytes();
            let sub_splits = sub_splitter.hf_splits(data);
            assert_tiles(data, &sub_splits);
            assert_token_partition(tokenizer, data, &sub_splits, None);
            let sub_tokens: Vec<u32> = sub_splits.iter().flat_map(|s| s.tokens.clone()).collect();
            assert_eq!(sub_tokens, split.tokens);
        });
    }

    #[test]
    fn resplitting_chunks_at_full_merge_level_is_lossless() {
        let data = read("tests/test_data/superlinear.txt");
        let splitter = hf_lite(paragraph_patterns(), Some(512), None, true, DEFAULT_MODEL);
        let sub_splitter = hf_lite(
            paragraph_patterns(),
            Some(512),
            Some(3),
            true,
            DEFAULT_MODEL,
        );

        let splits = splitter.hf_splits(&data);
        assert_tiles(&data, &splits);
        assert_resplit_is_lossless(&splitter.tokenizer.tokenizer, &sub_splitter, &splits);
    }

    /// The corpus check: split a file at each `merge_level`, verify tiling and token
    /// partition, then verify every chunk re-splits losslessly.
    fn check_corpus_file(path: &str, patterns: &[Vec<String>], merge_level: usize) {
        let data = read(path);
        let splitter = hf_lite(
            patterns.to_vec(),
            Some(512),
            Some(merge_level),
            true,
            DEFAULT_MODEL,
        );
        let sub_splitter = hf_lite(
            patterns.to_vec(),
            None,
            Some(patterns.len()),
            true,
            DEFAULT_MODEL,
        );
        let tokenizer = &splitter.tokenizer.tokenizer;

        let splits = splitter.hf_splits(&data);
        assert_tiles(&data, &splits);
        assert_token_partition(tokenizer, &data, &splits, Some(512));
        assert_resplit_is_lossless(tokenizer, &sub_splitter, &splits);
    }

    fn check_corpus(files: &[String], patterns: &[Vec<String>]) {
        tokenizers::utils::parallelism::set_parallelism(true);
        files.par_iter().for_each(|path| {
            for merge_level in 1..3 {
                check_corpus_file(path, patterns, merge_level);
            }
        });
    }

    #[test]
    fn icu_sentence_splits_of_superlinear_reencode() {
        let data = read("tests/test_data/superlinear.txt");
        let patterns = sentence_first_patterns("<ICU_SENT>");
        let splitter = hf_lite(patterns.clone(), Some(512), Some(2), true, DEFAULT_MODEL);
        let sub_splitter = hf_lite(patterns, None, Some(3), true, DEFAULT_MODEL);
        let tokenizer = &splitter.tokenizer.tokenizer;

        let splits = splitter.hf_splits(&data);
        assert_tiles(&data, &splits);
        assert_token_partition(tokenizer, &data, &splits, Some(512));
        assert_splits_reencode(tokenizer, &splits);
        assert_resplit_is_lossless(tokenizer, &sub_splitter, &splits);
    }

    #[test]
    fn llm_papers_corpus() {
        let files = text_files("tests/llm_papers_txt");
        check_corpus(&files, &sentence_first_patterns("<ICU_SENT>"));
    }

    #[test]
    #[ignore = "needs the Natural Questions corpus in data/dev and data/train (not committed)"]
    fn natural_questions_corpus() {
        let all_files = [text_files("data/dev"), text_files("data/train")].concat();
        // Every Nth file, so the sample is deterministic and about 3000 files.
        let step = (all_files.len() / 3000).max(1);
        let files: Vec<String> = all_files.into_iter().step_by(step).collect();

        check_corpus(&files, &patterns(&[&["\n\n"], &["\n"], &[". ", "!", "?"]]));
        check_corpus(&files, &sentence_first_patterns("<SENT>"));
    }

    // ---------------------------------------------------------------------------------
    // Normalization and sentence segmentation as the splitter relies on them
    // ---------------------------------------------------------------------------------

    #[test]
    fn chinese_splits_survive_normalization_roundtrip() {
        let original = String::from_utf8(read("tests/test_data/chinese_example01.txt")).unwrap();
        let normalizer = TextNormalizer::default();
        let normalized = normalizer.normalize(&original).unwrap();
        let patterns = patterns(&[&["\n\n"], &["\n"], &[".", ". ", ",", ", ", "、", "。"]]);

        let ws_splits = SplitterLiteConfig::new_ws(patterns.clone(), Some(512), None, true, false)
            .ws_splits(normalized.as_bytes());
        assert_tiles(normalized.as_bytes(), &ws_splits);

        let denormalized: usize = ws_splits
            .iter()
            .map(|s| normalizer.denormalize(&s.split_string).unwrap().len())
            .sum();
        assert_eq!(denormalized, original.len());

        let none_splits = SplitterLiteConfig::new_none(patterns, Some(512), None, false)
            .len_splits(original.as_bytes());
        assert_tiles(original.as_bytes(), &none_splits);
    }

    #[test]
    fn icu_sentences_tile_document_and_encode_independently() {
        let data = String::from_utf8(read("tests/test_data/superlinear.txt")).unwrap();
        let sentences = Segmenter::icu().sentences(&data).unwrap();

        let joined: String = sentences.concat();
        assert_eq!(joined, data);

        // Encoding sentence by sentence must give the document encoding: the splitter
        // relies on this to tokenize each piece once and never re-tokenize the whole.
        let tokenizer = tokenizer(DEFAULT_MODEL);
        let document = tokenizer.encode(data.as_str(), false).unwrap();
        let joined_ids: Vec<u32> = sentences
            .par_iter()
            .map(|s| {
                tokenizer
                    .encode(*s, false)
                    .unwrap()
                    .get_ids()
                    .to_vec()
            })
            .flatten()
            .collect();
        assert_eq!(joined_ids, document.get_ids());
    }

    /// The benches use `kitoken` for speed; it must produce the same ids as the HF tokenizer.
    #[test]
    fn kitoken_matches_hf_tokenizer() {
        use kitoken::{Definition, Kitoken, Processing};

        let data = String::from_utf8(read("tests/test_data/superlinear.txt")).unwrap();
        let mut definition =
            Definition::from_tokenizers_file(tokenizer_file(DEFAULT_MODEL)).unwrap();
        definition.config.processing = definition
            .config
            .processing
            .iter()
            .filter_map(|step| match step {
                Processing::Pad { .. } => None,
                Processing::Truncate {
                    stride, direction, ..
                } => Some(Processing::Truncate {
                    length: u32::MAX,
                    stride: *stride,
                    direction: *direction,
                }),
                other => Some(other.clone()),
            })
            .collect();
        let kitoken = Kitoken::from_definition(definition).unwrap();
        let hf = tokenizer(DEFAULT_MODEL);

        assert_eq!(
            kitoken.encode(data.as_str(), false).unwrap(),
            hf.encode(data.as_str(), false).unwrap().get_ids()
        );
        for sentence in Segmenter::icu().sentences(&data).unwrap() {
            assert_eq!(
                kitoken.encode(sentence, false).unwrap(),
                hf.encode(sentence, false).unwrap().get_ids(),
                "{sentence:?}"
            );
        }
    }
}
