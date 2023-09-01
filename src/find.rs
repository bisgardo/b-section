pub fn find<T, E>(
    lookup: impl Fn(i64) -> Result<T, E>,
    target: &T,
    lower_idx: i64,
    upper_idx: i64,
) -> Result<Option<T>, E> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use assert_matches::assert_matches; // use stdlib version once it's stable

    fn new_lookup(list: &[i64]) -> impl Fn(i64) -> Result<i64> + '_ {
        |idx| Ok(list[idx as usize])
    }

    #[test]
    fn cannot_find_element_in_empty_list() {
        let result = find(new_lookup(&[]), &0, 0, 0);
        assert_matches!(result, Ok(None));
    }
}
