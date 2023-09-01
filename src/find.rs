pub fn find<T: Ord, E>(
    lookup: impl Fn(i64) -> Result<T, E>,
    target: &T,
    mut lower_idx: i64, // inclusive
    mut upper_idx: i64, // inclusive
) -> Result<Option<T>, E> {
    while lower_idx <= upper_idx {
        let idx = (lower_idx + upper_idx) / 2;
        let val = lookup(idx)?;
        if val < *target {
            lower_idx = idx + 1;
        } else if val > *target {
            upper_idx = idx - 1;
        } else {
            return Ok(Some(val));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    /* HELPER */

    fn new_lookup(arr: &[i64]) -> impl Fn(i64) -> Result<i64, String> + '_ {
        |idx| {
            if idx < 0 {
                return Err(format!("negative index {}", idx));
            }
            let idx = idx as usize;
            if idx >= arr.len() {
                return Err(format!("index {} out of bounds", idx));
            }
            Ok(arr[idx])
        }
    }

    /* MISC */

    #[test]
    fn lookup_error_is_propagated() {
        assert_matches!(
            find(|_| Err("forget it"), &0, 0, 0),
            Err(msg) if msg == "forget it"
        );
    }

    #[test]
    fn finding_in_empty_array_fails() {
        assert_matches!(
            find(new_lookup(&[]), &0, 0, 0),
            Err(msg) if msg == "index 0 out of bounds"
        );
    }

    /* SINGLETON ARRAY */

    #[test]
    fn can_find_element_present_in_singleton_array() {
        assert_matches!(
            find(new_lookup(&[0]), &0, 0, 0),
            Ok(Some(0))
        );
    }

    #[test]
    fn cannot_find_element_not_present_in_singleton_array() {
        assert_matches!(
            find(new_lookup(&[0]), &1, 0, 0),
            Ok(None)
        );
    }

    /* TWO-ELEMENT ARRAY */

    #[test]
    fn can_find_elements_in_two_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &0, 0, 1),
            Ok(Some(0))
        );
        assert_matches!(
            find(new_lookup(&[0, 1]), &1, 0, 1),
            Ok(Some(1))
        );
    }

    #[test]
    fn cannot_find_elements_not_present_in_two_element_array() {
        let arr = &[0, 2];
        let ts = &[
            -1, // below first element
            1, // between elements
            3, // above last element
        ];
        for t in ts {
            assert_matches!(
                find(new_lookup(arr), t, 0, 1),
                Ok(None)
            );
        }
    }

    /* SINGLETON SUBSET OF TWO-ELEMENT ARRAY */

    #[test]
    fn can_find_element_in_singleton_subset_of_two_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &0, 0, 0),
            Ok(Some(0))
        );
        assert_matches!(
            find(new_lookup(&[0, 1]), &1, 1, 1),
            Ok(Some(1))
        );
    }

    #[test]
    fn cannot_find_element_not_present_in_singleton_subset_of_two_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &0, 1, 1),
            Ok(None)
        );
        assert_matches!(
            find(new_lookup(&[0, 1]), &1, 0, 0),
            Ok(None)
        );
    }

    /* LONGER ARRAYS */

    #[test]
    fn can_find_element_in_three_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1, 2]), &0, 0, 3),
            Ok(Some(0))
        );
        assert_matches!(
            find(new_lookup(&[0, 1, 2]), &1, 0, 3),
            Ok(Some(1))
        );
        assert_matches!(
            find(new_lookup(&[0, 1, 2]), &2, 0, 3),
            Ok(Some(2))
        );
    }

    #[test]
    fn can_find_element_in_four_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1, 2, 3]), &0, 0, 4),
            Ok(Some(0))
        );
        assert_matches!(
            find(new_lookup(&[0, 1, 2, 3]), &1, 0, 4),
            Ok(Some(1))
        );
        assert_matches!(
            find(new_lookup(&[0, 1, 2, 3]), &2, 0, 4),
            Ok(Some(2))
        );
        assert_matches!(
            find(new_lookup(&[0, 1, 2, 3]), &3, 0, 4),
            Ok(Some(3))
        );
    }
}
