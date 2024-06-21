#[derive(Default, PartialEq, Debug, Clone)]
pub struct TokensResults {
    pub ids: Vec<u32>,
    pub type_ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub offsets: Vec<(usize, usize)>,
    pub tokens: Vec<String>,
}

impl TokensResults {
    pub fn extend(&mut self, other: TokensResults) {
        self.ids.extend(other.ids);
        self.type_ids.extend(other.type_ids);
        self.attention_mask.extend(other.attention_mask);
        self.offsets.extend(other.offsets);
        self.tokens.extend(other.tokens);
    }
}
