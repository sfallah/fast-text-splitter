#[derive(Debug, Clone)]
pub struct SearchPattern {
    pub pattern: String,
    pub pattern_len: usize,
    pub is_whitespace: bool,
}

impl SearchPattern {
    pub fn new(pattern: String) -> Self {
        let pattern_len = pattern.len();
        let is_whitespace = pattern.trim().is_empty();
        Self {
            pattern,
            pattern_len,
            is_whitespace,
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