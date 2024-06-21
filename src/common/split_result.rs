use crate::common::split::Split;
use crate::common::tokens_result::TokensResults;
use std::fmt;

#[derive(PartialEq, Clone)]
pub struct SplitResults {
    pub split: Split,

    pub results: Option<TokensResults>,
    pub split_strings: String,
}

impl fmt::Debug for SplitResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SplitResults {{ split: {:?}, \n split_strings: {:?} }}",
            self.split, self.split_strings
        )
    }
}
