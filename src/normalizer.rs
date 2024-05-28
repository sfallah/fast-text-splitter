extern crate unicode_normalization;

use std::string::String;

use serde::{Deserialize, Serialize};
use unicode_categories::UnicodeCategories;
use unicode_normalization::UnicodeNormalization;

/// Checks whether a character is whitespace
fn is_whitespace(c: char) -> bool {
    // These are technically control characters but we count them as whitespace
    match c {
        '\t' | '\n' | '\r' => true,
        _ => c.is_whitespace(),
    }
}

/// Checks whether a character is a control character
fn is_control(c: char) -> bool {
    // These are technically control characters but we count them as whitespace
    match c {
        '\t' | '\n' | '\r' => false,
        // The definition of `is_control` here is quite large and contains also
        // Cc, Cf, Cn or Co
        // cf. https://unicode.org/reports/tr44/ (Table 12)
        _ => c.is_other(),
    }
}

/// Checks whether a character is chinese
/// This defines a "chinese character" as anything in the CJK Unicode block:
///   https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_(Unicode_block)
///
/// Note that the CJK Unicode block is NOT all Japanese and Korean characters,
/// despite its name. The modern Korean Hangul alphabet is a different block,
/// as is Japanese Hiragana and Katakana. Those alphabets are used to write
/// space-separated words, so they are not treated specially and handled
/// like for all of the other languages.
fn is_chinese_char(c: char) -> bool {
    matches!(
        c as usize,
        0x4E00..=0x9FFF |
        0x3400..=0x4DBF |
        0x20000..=0x2A6DF |
        0x2A700..=0x2B73F |
        0x2B740..=0x2B81F |
        0x2B920..=0x2CEAF |
        0xF900..=0xFAFF |
        0x2F800..=0x2FA1F
    )
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub struct TextNormalizer {
    /// Whether to do the bert basic cleaning:
    ///   1. Remove any control characters
    ///   2. Replace all sorts of whitespace by the classic one ` `
    pub clean_text: bool,
    /// Whether to put spaces around chinese characters so they get split
    pub handle_chinese_chars: bool,
    /// Whether to strip accents
    pub strip_accents: Option<bool>,
    /// Whether to lowercase the input
    pub lowercase: bool,
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self {
            clean_text: true,
            handle_chinese_chars: true,
            strip_accents: None,
            lowercase: true,
        }
    }
}

impl TextNormalizer {
    pub fn new(
        clean_text: bool,
        handle_chinese_chars: bool,
        strip_accents: Option<bool>,
        lowercase: bool,
    ) -> Self {
        Self {
            clean_text,
            handle_chinese_chars,
            strip_accents,
            lowercase,
        }
    }

    fn do_clean_text(&self, normalized: &String) -> String {
        normalized.chars()
            .filter(|c| !(*c as usize == 0 || *c as usize == 0xfffd || is_control(*c)))
            //.map(|c| if is_whitespace(c) { ' ' } else { c })
            .collect::<String>()
    }

    fn do_handle_chinese_chars(&self, data: &String) -> String{
        let mut normalized = String::new();
        data.chars().for_each(|c| {
            if is_chinese_char(c) {
                normalized.push(' ');
                normalized.push(c);
                normalized.push(' ');
            } else {
                normalized.push(c);
            }
        });
        normalized
    }


    fn do_strip_accents(&self, normalized: &String) -> String {
        normalized.to_owned().nfd().filter(|c| !c.is_mark_nonspacing()).collect::<String>()
    }

    fn do_lowercase(&self, normalized: &String) -> String {
        normalized.to_lowercase()
    }
}

impl TextNormalizer {
    pub fn normalize(&self, data: &String) -> anyhow::Result<String> {
        let mut normalized = data.clone();
        if self.clean_text {
            normalized = self.do_clean_text(&normalized);
        }
        if self.handle_chinese_chars {
            normalized = self.do_handle_chinese_chars(&normalized);
        }

        let strip_accents = self.strip_accents.unwrap_or(self.lowercase);

        if strip_accents {
            normalized = self.do_strip_accents(&normalized);
        }

        if self.lowercase {
            normalized = self.do_lowercase(&normalized);
        }

        Ok(normalized)
    }
}
