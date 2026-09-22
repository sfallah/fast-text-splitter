#[derive(Debug, Clone)]
pub struct SearchPattern {
    pub pattern: String,
    pub pattern_len: usize,
    pub is_whitespace: bool,
    pub is_sentence: bool,
    pub is_icu_sentence: bool,
}

impl SearchPattern {
    pub fn new(pattern: String) -> Self {
        let pattern_len = pattern.len();
        let is_whitespace = pattern.trim().is_empty();
        Self {
            pattern: pattern.clone(),
            pattern_len,
            is_whitespace,
            is_sentence: pattern == "<SENT>",
            is_icu_sentence: pattern == "<ICU_SENT>",
        }
    }

    pub fn len(&self) -> usize {
        self.pattern_len
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.pattern.as_bytes()
    }
    pub fn is_single_byte(&self) -> bool {
        self.pattern_len == 1
    }
}
