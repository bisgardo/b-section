pub enum Snap {
    Backwards, Forwards
}

pub fn find<T: PartialOrd, E>(
    lookup: impl Fn(i64) -> Result<T, E>,
    target: &T,
    mut lower_idx: i64, // inclusive
    mut upper_idx: i64, // inclusive
    snap: Option<Snap>,
) -> Result<Option<T>, E> {
    let mut res = None;
    while lower_idx <= upper_idx {
        let idx = (lower_idx + upper_idx) / 2;
        let val = lookup(idx)?;
        if val < *target {
            if let Some(Snap::Backwards) = snap {
                res = Some(val);
            }
            lower_idx = idx + 1;
        } else if val > *target {
            if let Some(Snap::Forwards) = snap {
                res = Some(val);
            }
            upper_idx = idx - 1;
        } else {
            return Ok(Some(val));
        }
    }
    Ok(res)
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
            find(|_| Err("forget it"), &0, 0, 0, None),
            Err(msg) if msg == "forget it"
        );
    }

    #[test]
    fn finding_in_empty_array_fails() {
        assert_matches!(
            find(new_lookup(&[]), &0, 0, 0, None),
            Err(msg) if msg == "index 0 out of bounds"
        );
    }

    /* SINGLETON ARRAY */

    #[test]
    fn can_find_element_present_in_singleton_array() {
        assert_matches!(
            find(new_lookup(&[0]), &0, 0, 0, None),
            Ok(Some(0))
        );
    }

    #[test]
    fn cannot_find_element_not_present_in_singleton_array() {
        assert_matches!(
            find(new_lookup(&[0]), &1, 0, 0, None),
            Ok(None)
        );
    }

    /* TWO-ELEMENT ARRAY */

    #[test]
    fn can_find_elements_in_two_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &0, 0, 1, None),
            Ok(Some(0))
        );
        assert_matches!(
            find(new_lookup(&[0, 1]), &1, 0, 1, None),
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
                find(new_lookup(arr), t, 0, 1, None),
                Ok(None)
            );
        }
    }

    /* SINGLETON SUBSET OF TWO-ELEMENT ARRAY */

    #[test]
    fn can_find_element_in_singleton_subset_of_two_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &0, 0, 0, None),
            Ok(Some(0))
        );
        assert_matches!(
            find(new_lookup(&[0, 1]), &1, 1, 1, None),
            Ok(Some(1))
        );
    }

    #[test]
    fn cannot_find_element_not_present_in_singleton_subset_of_two_element_array() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &0, 1, 1, None),
            Ok(None)
        );
        assert_matches!(
            find(new_lookup(&[0, 1]), &1, 0, 0, None),
            Ok(None)
        );
    }

    /* LONGER ARRAYS */

    #[test]
    fn can_find_element_in_three_element_array() {
        let arr = &[0, 1, 2];
        for v in arr {
            assert_matches!(
                find(new_lookup(arr), v, 0, 2, None),
                Ok(Some(r)) if r == *v
            );
        }
    }

    #[test]
    fn can_find_element_in_four_element_array_using_any_snap() {
        let arr = &[0, 1, 2, 3];
        for v in arr {
            assert_matches!(
                find(new_lookup(arr), v, 0, 3, None),
                Ok(Some(r)) if r == *v
            );
            assert_matches!(
                find(new_lookup(arr), v, 0, 3, Some(Snap::Backwards)),
                Ok(Some(r)) if r == *v
            );
            assert_matches!(
                find(new_lookup(arr), v, 0, 3, Some(Snap::Forwards)),
                Ok(Some(r)) if r == *v
            );
        }
    }

    #[test]
    fn can_find_element_before_unmatched_target_using_backwards_snap() {
        assert_matches!(
            find(new_lookup(&[0, 2]), &1, 0, 1, Some(Snap::Backwards)),
            Ok(Some(0))
        );
    }

    #[test]
    fn can_find_element_after_unmatched_target_using_forwards_snap() {
        assert_matches!(
            find(new_lookup(&[0, 2]), &1, 0, 1, Some(Snap::Forwards)),
            Ok(Some(2))
        );
    }

    #[test]
    fn can_find_first_element_for_target_less_than_first_element_using_forwards_snap() {
        assert_matches!(
            find(new_lookup(&[1, 2]), &0, 0, 1, Some(Snap::Forwards)),
            Ok(Some(1))
        );
    }

    #[test]
    fn can_find_last_element_for_target_greater_than_last_element_using_backwards_snap() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &2, 0, 1, Some(Snap::Backwards)),
            Ok(Some(1))
        );
    }

    #[test]
    fn cannot_find_element_before_target_less_than_first_element_using_backwards_snap() {
        assert_matches!(
            find(new_lookup(&[1, 2]), &0, 0, 1, Some(Snap::Backwards)),
            Ok(None)
        );
    }

    #[test]
    fn cannot_find_element_after_target_greater_than_last_element_using_forwards_snap() {
        assert_matches!(
            find(new_lookup(&[0, 1]), &2, 0, 1, Some(Snap::Forwards)),
            Ok(None)
        );
    }
}
