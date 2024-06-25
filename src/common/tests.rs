#[cfg(test)]
mod tests {

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
}
