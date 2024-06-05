use aho_corasick::Span;
use fast_text_splitter::splitter::split;
use std::fs;
use std::str::from_utf8;

#[test]
fn pattern_split_superlinear_test() {
    let data_path = "tests/test_data/superlinear_modified.txt";

    let binding = fs::read_to_string(data_path).unwrap();
    let data = binding.as_bytes();

    let patterns = vec!["\n\n", "\n", "."];

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
    let reconsted = tree.reconstruct();
    assert_eq!(from_utf8(&data).unwrap(), reconsted);

    let merged_0 = tree.merge(0, false);
    let splits_len_sum: usize = merged_0.iter().map(|span| span.len()).sum();
    assert_eq!(data.len(), splits_len_sum);

    let merged_1 = tree.merge(1, false);
    let merged_2 = tree.merge(2, false);
    println!("Merged 0: {:?}", merged_0.len());
    println!("Merged 1: {:?}", merged_1.len());
    println!("Merged 2: {:?}", merged_2.len());

    let merged_spans_filtered: Vec<_> = merged_1.iter().filter(|span| !span.is_empty()).collect();

    println!("Merged Spans: {:?}", merged_spans_filtered.len());
    for span in merged_spans_filtered {
        println!("{:?}", from_utf8(&data[span.start..span.end]).unwrap());
    }
}
