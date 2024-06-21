#[cfg(test)]
mod tests {
    use crate::common::tokens_result::TokensResults;

    #[test]
    fn tokens_result_extend_test() {
        let tk_res = TokensResults::default();
        assert!(tk_res.ids.is_empty());
        assert!(tk_res.type_ids.is_empty());
        assert!(tk_res.attention_mask.is_empty());
        assert!(tk_res.offsets.is_empty());
    }

    #[test]
    fn vec_extend_test() {
        let mut empty_vec = vec![];
        let vec1 = vec![1, 2, 3];
        empty_vec.extend(vec1);
        assert_eq!(empty_vec, vec![1, 2, 3]);

        let mut vec2 = vec![4, 5, 6];
        let empty_vec2: Vec<i32> = vec![];
        vec2.extend(empty_vec2);
        assert_eq!(vec2, vec![4, 5, 6]);
    }

    #[test]
    fn tokens_result_extend_test2() {
        let mut tk_res = TokensResults::default();
        let mut tk_res2 = TokensResults::default();
        tk_res2.ids = vec![1, 2, 3];
        tk_res2.type_ids = vec![4, 5, 6];
        tk_res2.attention_mask = vec![7, 8, 9];
        tk_res2.offsets = vec![(1, 2), (3, 4), (5, 6)];
        tk_res.extend(tk_res2);
        assert_eq!(tk_res.ids, vec![1, 2, 3]);
        assert_eq!(tk_res.type_ids, vec![4, 5, 6]);
        assert_eq!(tk_res.attention_mask, vec![7, 8, 9]);
        assert_eq!(tk_res.offsets, vec![(1, 2), (3, 4), (5, 6)]);
    }

    #[test]
    fn tokens_result_default_test() {
        let tk_res = TokensResults::default();
        assert!(tk_res.ids.is_empty());
        assert!(tk_res.type_ids.is_empty());
        assert!(tk_res.attention_mask.is_empty());
        assert!(tk_res.offsets.is_empty());
    }
}