use aho_corasick::Span;
use fast_text_splitter::common::span;
use fast_text_splitter::pattern_search::{find_pattern, search_patterns, span_to_string};


#[test]
fn strange_case_test() -> anyhow::Result<()> {
    let pattern = "\n";
    let data = "In fact, the correlation. Between superlinear.\nReturns and inequality is so strong that it yields.";
    let result = find_pattern(&vec![pattern], data.as_bytes(), span(0, data.len()));
    assert_eq!(result.splits.len(), 2);

    let data1 = "Returns and inequality is so strong that it yields.".as_bytes();
    let pattern1 = ".";
    let result1 = find_pattern(&vec![pattern1], data1, span(0, data1.len()));
    assert_eq!(result1.splits.len(), 1);

    Ok(())
}

#[test]
fn search_empty_test() -> anyhow::Result<()> {
    let pattern = "\n";
    let data = "";
    let result = find_pattern(&vec![pattern], data.as_bytes(), span(0, data.len()));
    assert_eq!(result.splits.len(), 1);
    assert_eq!(result.splits[0].data(), "".to_string());
    Ok(())
}

#[test]
fn span_to_string_test() -> anyhow::Result<()> {
    let data = "Hello, you all! How are you?";
    let empty_span = Span { start: 5, end: 5 };
    let empty_str = span_to_string(data, empty_span);
    assert_eq!(empty_str, Some("".to_string()));

    let valid_span = Span { start: 5, end: 10 };
    let valid_str = span_to_string(data, valid_span);
    assert_eq!(valid_str, Some(", you".to_string()));

    let data_span = Span {
        start: 0,
        end: data.len(),
    };
    let data_str = span_to_string(data, data_span);
    assert_eq!(data_str, Some(data.to_string()));

    let wrong_span = Span { start: 5, end: 2 };
    let wrong_str = span_to_string(data, wrong_span);
    assert_eq!(wrong_str, None);

    let end_span = Span {
        start: data.len() - 3,
        end: data.len() + 1,
    };
    let end_str = span_to_string(data, end_span);
    assert_eq!(end_str, None);

    Ok(())
}

#[test]
fn no_matches_test() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let data1 = "Hello, you all! How are you?";

    let no_match_res = find_pattern(&vec![pattern], data1.as_bytes(), span(0, data1.len()));
    assert!(!no_match_res.matched);

    assert_eq!(no_match_res.data()[0], data1.to_string());
    Ok(())
}

#[test]
fn empty_data_test() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let data1 = "";

    let result = find_pattern(&vec![pattern], data1.as_bytes(), span(0, data1.len()));
    assert!(!result.matched);

    Ok(())
}

#[test]
fn single_ends_test() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let data1 = "\n\nHello, you all! How are you?".as_bytes();
    let result1 = find_pattern(&vec![pattern], data1, span(0, data1.len()));

    assert_eq!(result1.splits.len(), 2);
    assert_eq!(result1.splits[0].span, Span { start: 0, end: 0 });
    assert_eq!(result1.splits[0].stride, 1);
    assert_eq!(result1.splits[0].data(), "".to_string());
    assert_eq!(result1.splits[0].reconstruct(), "\n\n".to_string());

    assert_eq!(result1.splits[1].span.len(), data1.len() - 2);
    assert_eq!(result1.splits[1].stride, 0);
    assert_eq!(
        result1.splits[1].data(),
        "Hello, you all! How are you?".to_string()
    );
    assert_eq!(
        result1.splits[1].reconstruct(),
        "Hello, you all! How are you?".to_string()
    );

    let data2 = "Hello, you all! How are you?\n\n".as_bytes();
    let result2 = find_pattern(&vec![pattern], data2, span(0, data2.len()));
    assert_eq!(result2.splits.len(), 1);
    assert_eq!(result2.splits[0].span.len(), data2.len() - 2);
    assert_eq!(result2.splits[0].stride, 1);
    assert_eq!(
        result2.splits[0].data(),
        "Hello, you all! How are you?".to_string()
    );
    assert_eq!(
        result2.splits[0].reconstruct(),
        "Hello, you all! How are you?\n\n".to_string()
    );

    Ok(())
}

#[test]
fn offseted_single_ends_test() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let before = "<from before NOT IN SEARCH>\n\n";
    let before_len = before.len();
    let data_1 = "\n\nHello, you all! How are you?";
    let binding = before.to_owned() + data_1;
    let data1 = binding.as_bytes();
    let result1 = find_pattern(&vec![pattern], data1, span(before_len, data1.len()));

    assert_eq!(result1.splits.len(), 2);
    assert_eq!(
        result1.splits[0].span,
        Span {
            start: before_len,
            end: before_len,
        }
    );
    assert_eq!(result1.splits[0].stride, 1);
    assert_eq!(result1.splits[0].data(), "".to_string());
    assert_eq!(result1.splits[0].reconstruct(), "\n\n".to_string());

    assert_eq!(result1.splits[1].span.len(), data_1.len() - 2);
    assert_eq!(result1.splits[1].stride, 0);
    assert_eq!(
        result1.splits[1].data(),
        "Hello, you all! How are you?".to_string()
    );
    assert_eq!(
        result1.splits[1].reconstruct(),
        "Hello, you all! How are you?".to_string()
    );

    let data_2 = "Hello, you all! How are you?\n\n";
    let binding = before.to_owned() + data_2;
    let data2 = binding.as_bytes();
    let result2 = find_pattern(&vec![pattern], data2, span(before_len, data2.len()));
    assert_eq!(result2.splits.len(), 1);
    assert_eq!(result2.splits[0].span.len(), data_2.len() - 2);
    assert_eq!(result2.splits[0].stride, 1);
    assert_eq!(
        result2.splits[0].data(),
        "Hello, you all! How are you?".to_string()
    );
    assert_eq!(
        result2.splits[0].reconstruct(),
        "Hello, you all! How are you?\n\n".to_string()
    );

    Ok(())
}

#[test]
fn single_middle_test() -> anyhow::Result<()> {
    let pattern = "\n\n";

    let data4 = "Hello, you all!\n\nHow are you ?".as_bytes();
    let result4 = find_pattern(&vec![pattern], data4, span(0, data4.len()));
    assert_eq!(result4.splits.len(), 2);
    assert_eq!(result4.splits[0].stride, 1);
    assert_eq!(result4.splits[1].stride, 0);
    assert_eq!(result4.splits[0].data(), "Hello, you all!".to_string());
    assert_eq!(result4.splits[1].data(), "How are you ?".to_string());

    Ok(())
}

#[test]
fn test_data_nlnl() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let file_path = "tests/splitter_test_data/data_nlnl_01.txt";
    let binding = std::fs::read(file_path).unwrap();
    let data = binding.as_slice();

    let result = find_pattern(&vec![pattern], data, span(0, data.len()));
    assert_eq!(result.splits.len(), 8);

    for split in result.splits.iter() {
        println!("{:?}", split.reconstruct());
    }

    Ok(())
}

#[test]
fn find_multiple_patterns_ends_test() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let data1 = "\n\nHello, you all! How are you? \n\n".as_bytes();
    let result1 = find_pattern(&vec![pattern], data1, span(0, data1.len()));

    assert_eq!(result1.splits.len(), 2);
    assert_eq!(result1.splits[0].stride, 1);
    assert_eq!(result1.splits[0].data(), "".to_string());
    assert_eq!(result1.splits[1].stride, 1);
    assert_eq!(
        result1.splits[1].data(),
        "Hello, you all! How are you? ".to_string()
    );

    let data2 = "\n\n\n\nHello, you all! How are you? \n\n\n\n".as_bytes();
    let result2 = find_pattern(&vec![pattern], data2, span(0, data2.len()));
    assert_eq!(result2.splits.len(), 4);
    assert_eq!(result2.splits[0].stride, 1);
    assert_eq!(result2.splits[0].data(), "".to_string());
    assert_eq!(result2.splits[1].stride, 1);
    assert_eq!(result2.splits[1].data(), "".to_string());
    assert_eq!(result2.splits[2].stride, 1);
    assert_eq!(
        result2.splits[2].data(),
        "Hello, you all! How are you? ".to_string()
    );
    assert_eq!(result2.splits[3].stride, 1);
    assert_eq!(result2.splits[3].data(), "".to_string());
    Ok(())
}

#[test]
fn search_pattern_test() -> anyhow::Result<()> {
    let patterns = &vec!["\n\n"];
    let data = "Hello, you all!\n\n How are you? \n\n".as_bytes();
    let data_span = span(0, data.len());
    let matches = search_patterns(patterns, data, data_span);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].pattern, "\n\n");
    assert_eq!(matches[0].span, span(15, 17));
    assert_eq!(matches[1].pattern, "\n\n");
    assert_eq!(matches[1].span, span(31, 33));

    Ok(())
}

#[test]
fn search_patterns_test() -> anyhow::Result<()> {
    let patterns = vec![".", "!", "?"];
    let data = "Hello, you all! How are you? Nice to be here.".as_bytes();
    let data_span = span(0, data.len());
    let matches = search_patterns(&patterns, data, data_span);
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].pattern, "!");
    assert_eq!(matches[0].span, span(14, 15));
    assert_eq!(matches[1].pattern, "?");
    assert_eq!(matches[1].span, span(27, 28));
    assert_eq!(matches[2].pattern, ".");
    assert_eq!(matches[2].span, span(44, 45));

    let data = "Hello, you all!\n\n How are you? \n\n".as_bytes();
    let patterns = vec!["\n\n", "\n"];
    let matches = search_patterns(&patterns, data, span(0, data.len()));
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].pattern, "\n\n");
    assert_eq!(matches[0].span, span(15, 17));
    assert_eq!(matches[1].pattern, "\n\n");
    assert_eq!(matches[1].span, span(31, 33));

    let data = "Hello, you all!\n\n How are you? \n\n".as_bytes();
    let patterns = vec!["\n", "\n\n"];
    let matches = search_patterns(&patterns, data, span(0, data.len()));
    assert_eq!(matches.len(), 4);
    assert_eq!(matches[0].pattern, "\n");
    assert_eq!(matches[0].span, span(15, 16));
    assert_eq!(matches[1].pattern, "\n");
    assert_eq!(matches[1].span, span(16, 17));
    assert_eq!(matches[2].pattern, "\n");
    assert_eq!(matches[2].span, span(31, 32));
    assert_eq!(matches[3].pattern, "\n");
    assert_eq!(matches[3].span, span(32, 33));

    Ok(())
}

#[test]
fn find_multiple_patterns_test() -> anyhow::Result<()> {
    let pattern = "\n\n";
    let data1 = "Hello, you all!\n\n How are you? \n\n".as_bytes();

    let result1 = find_pattern(&vec![pattern], data1, span(0, data1.len()));
    assert_eq!(result1.splits.len(), 2);
    assert_eq!(result1.splits[0].stride, 1);
    assert_eq!(result1.splits[0].data(), "Hello, you all!".to_string());
    assert_eq!(result1.splits[1].stride, 1);
    assert_eq!(result1.splits[1].data(), " How are you? ".to_string());

    let data2 = "Hello, you all!\n\n How are you? \n\n Nice to meet you all!".as_bytes();
    let result2 = find_pattern(&vec![pattern], data2, span(0, data2.len()));
    assert_eq!(result2.splits.len(), 3);
    assert_eq!(result2.splits[0].stride, 1);
    assert_eq!(result2.splits[0].data(), "Hello, you all!".to_string());
    assert_eq!(result2.splits[1].stride, 1);
    assert_eq!(result2.splits[1].data(), " How are you? ".to_string());
    assert_eq!(result2.splits[2].stride, 0);
    assert_eq!(
        result2.splits[2].data(),
        " Nice to meet you all!".to_string()
    );

    let data3 = "\n\nHello, you all!\n\n How are you? \n\n Nice to meet you all!\n\n I hope you are all doing well!".as_bytes();
    let result2 = find_pattern(&vec![pattern], data3, span(0, data3.len()));
    assert_eq!(result2.splits.len(), 5);
    assert_eq!(result2.splits[0].stride, 1);
    assert_eq!(result2.splits[0].data(), "".to_string());
    assert_eq!(result2.splits[1].stride, 1);
    assert_eq!(result2.splits[1].data(), "Hello, you all!".to_string());
    assert_eq!(result2.splits[2].stride, 1);
    assert_eq!(result2.splits[2].data(), " How are you? ".to_string());
    assert_eq!(result2.splits[3].stride, 1);
    assert_eq!(
        result2.splits[3].data(),
        " Nice to meet you all!".to_string()
    );
    assert_eq!(result2.splits[4].stride, 0);
    assert_eq!(
        result2.splits[4].data(),
        " I hope you are all doing well!".to_string()
    );

    println!("{:?}", result2);

    let data4 =
        "Hello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n".as_bytes();
    let result3 = find_pattern(&vec![pattern], data4, span(0, data4.len()));
    assert_eq!(result3.splits.len(), 4);
    assert_eq!(result3.splits[0].stride, 1);
    assert_eq!(result3.splits[0].data(), "Hello, you all!".to_string());
    assert_eq!(result3.splits[1].stride, 1);
    assert_eq!(result3.splits[1].data(), " How are you? ".to_string());
    assert_eq!(result3.splits[2].stride, 1);
    assert_eq!(result3.splits[2].data(), "".to_string());
    assert_eq!(result3.splits[3].stride, 1);
    assert_eq!(
        result3.splits[3].data(),
        " Nice to meet you all!".to_string()
    );

    let data5 =
        "\n\n\n\nHello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n\n\n"
            .as_bytes();
    let result4 = find_pattern(&vec![pattern], data5, span(0, data5.len()));
    assert_eq!(result4.splits.len(), 7);
    assert_eq!(result4.splits[0].stride, 1);
    assert_eq!(result4.splits[0].data(), "".to_string());
    assert_eq!(result4.splits[1].stride, 1);
    assert_eq!(result4.splits[1].data(), "".to_string());
    assert_eq!(result4.splits[2].stride, 1);
    assert_eq!(result4.splits[2].data(), "Hello, you all!".to_string());
    assert_eq!(result4.splits[3].stride, 1);
    assert_eq!(result4.splits[3].data(), " How are you? ".to_string());
    assert_eq!(result4.splits[4].stride, 1);
    assert_eq!(result4.splits[4].data(), "".to_string());
    assert_eq!(result4.splits[5].stride, 1);
    assert_eq!(
        result4.splits[5].data(),
        " Nice to meet you all!".to_string()
    );
    assert_eq!(result4.splits[6].stride, 1);
    assert_eq!(result4.splits[6].data(), "".to_string());

    Ok(())
}

#[test]
fn reconstruct_test() -> anyhow::Result<()> {
    let pattern = &vec!["\n\n"];
    let data1 = "Hello, you all!\n\n How are you? \n\n";
    let result1 = find_pattern(pattern, data1.as_bytes(), span(0, data1.len()));
    let reconsted1 = result1.reconstruct();
    assert_eq!(reconsted1, data1);

    let data2 = "Hello, you all!\n\n How are you? \n\n Nice to meet you all!";
    let result2 = find_pattern(pattern, data2.as_bytes(), span(0, data2.len()));
    let reconsted2 = result2.reconstruct();
    assert_eq!(reconsted2, data2);

    let data3 =
        "\n\n\n\nHello, you all!\n\n How are you? \n\n\n\n Nice to meet you all!\n\n\n\n";

    let result3 = find_pattern(pattern, data3.as_bytes(), span(0, data3.len()));
    let reconsted3 = result3.reconstruct();
    assert_eq!(reconsted3, data3);

    let data4 = "\n\nHello, you all! How are you?";
    let result4 = find_pattern(pattern, data4.as_bytes(), span(0, data4.len()));
    let reconsted4 = result4.reconstruct();
    assert_eq!(reconsted4, data4);

    let data5 = "Hello, you all! How are you?";
    let result5 = find_pattern(pattern, data5.as_bytes(), span(0, data5.len()));
    let reconsted5 = result5.reconstruct();
    assert_eq!(reconsted5, data5);

    let data6 = "Hello, you all! How are you?\n\n";
    let result6 = find_pattern(pattern, data6.as_bytes(), span(0, data6.len()));
    let reconsted6 = result6.reconstruct();
    assert_eq!(reconsted6, data6);

    Ok(())
}