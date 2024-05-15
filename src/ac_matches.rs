use aho_corasick::{AhoCorasick, Match, MatchKind, PatternID, Span};

pub fn init_aho_corasick(patterns: Option<Vec<String>>) -> anyhow::Result<AhoCorasick> {
    let default_patterns = vec!["\n\n".to_string(), "\n".to_string()];
    let ac_patterns = match patterns {
        Some(patterns) => if patterns.is_empty() {
            default_patterns
        } else {
            patterns
        },
        None => default_patterns,
    };
   let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(ac_patterns)?;
    Ok(ac)
}

#[inline]
pub fn match_offsets(
    matches: &[Match],
    match_span: &Span,
    data_len: usize,
) -> anyhow::Result<(usize, usize)> {
    let start = if match_span.start > 0 {
        matches.get(match_span.start).unwrap().start()
    } else {
        0
    };
    let end = if match_span.end < matches.len() {
        matches.get(match_span.end).unwrap().start()
    } else {
        data_len
    };
    Ok((start, end))
}

#[inline]
pub fn matches_spans(matches: &[Match], pattern_id: usize) -> Vec<Span> {
    let indices = matches_indices(matches, pattern_id);
    matches_indices_to_spans(&indices, matches.len())
}

fn matches_indices(matches: &[Match], pattern_id: usize) -> Vec<usize> {
    matches
        .iter()
        .enumerate()
        .filter(|(_, mat)| mat.pattern() == PatternID::must(pattern_id))
        .map(|(i, _)| i)
        .collect()
}

fn matches_indices_to_spans(indices: &[usize], ln: usize) -> Vec<Span> {
    let mut spans = Vec::new();
    if indices.is_empty() {
        spans.push(Span { start: 0, end: 0 });
        return spans;
    }
    let mut start = indices[0];

    if start > 0 {
        spans.push(Span {
            start: 0,
            end: start,
        });
    }

    indices.iter().skip(1).for_each(|&idx| {
        spans.push(Span { start, end: idx });
        start = idx;
    });

    if start < ln {
        spans.push(Span { start, end: ln });
    }
    spans
}

#[inline(always)]
pub fn next_match_pos(matches: &[Match], mt_span: Span, pattern_id: usize) -> Option<usize> {
    matches
        .iter()
        .skip(mt_span.start)
        .take(mt_span.len())
        .position(|mat| mat.pattern() == PatternID::must(pattern_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span;

    #[test]
    fn init_aho_corasick_test() -> anyhow::Result<()> {
        let ac = init_aho_corasick(
            Some(vec!["something".to_string(), "else".to_string()]))?;
        assert_eq!(ac.patterns_len(), 2);
        Ok(())
    }

    #[test]
    fn matches_spans_test() -> anyhow::Result<()> {
        let data = "Hello, you all! \n How are you ?\n\n I am fine. Nice to meet you all insecure!";
        let ac = init_aho_corasick(None)?;

        let matches: Vec<_> = ac.find_iter(data).collect();
        println!("Matches: {:?}", matches);
        let indices = matches_indices(&matches, 0);
        println!("Indices: {:?}", indices);
        let spans = matches_indices_to_spans(&indices, matches.len());
        println!("Spans: {:?}", spans);
        Ok(())
    }

    #[test]
    fn match_pos_test() -> anyhow::Result<()> {
        let data = "Hello, you all! \n How are you ?\n\n I am fine. Nice to meet you all insecure!";
        let ac = init_aho_corasick(None)?;

        let matches: Vec<_> = ac.find_iter(data).collect();
        assert_eq!(matches.len(), 2);
        let pos = next_match_pos(&matches, span(0, matches.len()), 0);
        assert_eq!(pos, Some(1));
        Ok(())
    }

    #[test]
    fn no_matches_test() -> anyhow::Result<()>{
        let data = "Hello, you all! How are you ? I am fine. Nice to meet you all insecure!";
        let ac = init_aho_corasick(None)?;

        let matches: Vec<_> = ac.find_iter(data).collect();
        assert_eq!(matches.len(), 0);
        let mt_spans = matches_spans(&matches, 0);
        assert_eq!(mt_spans.len(), 1);
        assert_eq!(mt_spans[0], span(0, 0));
        Ok(())
    }
}
