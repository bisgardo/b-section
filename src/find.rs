#[derive(Clone)]
pub enum Snap {
    Downwards,
    Upwards,
}

// TODO: Can name types/cases after what the comparisons represent?
//       'True' case means that the value was not matches and that the search space has to be limited as a result.
//       And the "snap" value in that case determines whether to store the value as a (temporary) result anyway.
pub enum CmpResult {
    /// Inequality is satisfied. I.e. the checked value is *not* matched.
    True {
        /// Determines whether the non-matching value should be stored as a preliminary result.
        keep: bool,
    },
    False,
}

pub trait FindOrd<T> {
    fn lt(&self, t: &T) -> CmpResult;
    fn gt(&self, t: &T) -> CmpResult;
}

/// Let all PartialOrd types (of self) trivially implement FindOrd.
impl<T: PartialOrd<T>> FindOrd<T> for T {
    fn lt(&self, t: &T) -> CmpResult {
        if self.lt(t) {
            CmpResult::True { keep: false }
        } else {
            CmpResult::False
        }
    }

    fn gt(&self, t: &T) -> CmpResult {
        if self.gt(t) {
            CmpResult::True { keep: false }
        } else {
            CmpResult::False
        }
    }
}

#[derive(Debug)]
pub struct Element<T> {
    pub value: T,
    pub index: i64,
}

/// Result of searching for a value within the specified limits.
/// The bounds of the range defined by the limits are well defined even if no value was found:
/// In that case, the bounds are defined by a single value(?).
#[derive(Debug)]
pub struct FindResult<T> {
    /// Value satisfying the limits.
    pub element: Option<Element<T>>,
    /// Index of last inspected value that is below the lower limit (or `lower_idx`).
    pub last_lower_idx: i64,
    /// Index of last inspected value that is above the upper limit (or `upper_idx`).
    pub last_upper_idx: i64,
}

// TODO: Generify the index type (should be 'usize' for arrays).
pub fn find<T, E>(
    lookup: &impl Fn(i64) -> Result<T, E>,
    target: &dyn FindOrd<T>,
    mut lower_idx: i64, // inclusive
    mut upper_idx: i64, // inclusive
) -> Result<FindResult<T>, E> {
    let mut res = None;
    while lower_idx <= upper_idx {
        let idx = (lower_idx + upper_idx) / 2;
        let val = lookup(idx)?;
        if let CmpResult::True { keep } = target.gt(&val) {
            // val < target
            if keep {
                res = Some(Element { value: val, index: idx });
            }
            lower_idx = idx + 1;
            continue;
        }
        if let CmpResult::True { keep } = target.lt(&val) {
            // val > target
            if keep {
                res = Some(Element { value: val, index: idx });
            }
            upper_idx = idx - 1;
            continue;
        }
        // val matches target
        res = Some(Element { value: val, index: idx });
        break;
    }
    Ok(FindResult {
        element: res,
        last_lower_idx: lower_idx,
        last_upper_idx: upper_idx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::helpers::*;
    use assert_matches::assert_matches; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    fn find_value<T, E>(
        lookup: impl Fn(i64) -> Result<T, E>,
        target: &impl FindOrd<T>,
        lower_idx: i64, // inclusive
        upper_idx: i64, // inclusive
    ) -> Result<Option<T>, E> {
        let r = find(&lookup, target, lower_idx, upper_idx)?;
        Ok(r.element.map(|v| v.value))
    }

    /* MISC */

    #[test]
    fn lookup_error_is_propagated() {
        assert_matches!(
            find_value(|_| Err("forget it"), &0, 0, 0),
            Err(msg) if msg == "forget it"
        );
    }

    #[test]
    fn finding_in_empty_array_fails() {
        assert_matches!(
            find_value(new_lookup(&[]), &0, 0, 0),
            Err(msg) if msg == "index 0 out of bounds"
        );
    }

    /* SINGLETON ARRAY */

    #[test]
    fn can_find_element_present_in_singleton_array() {
        assert_matches!(
            find_value(new_lookup(&[0]), &0, 0, 0),
            Ok(Some(0))
        );
    }

    #[test]
    fn cannot_find_element_not_present_in_singleton_array() {
        assert_matches!(
            find_value(new_lookup(&[0]), &1, 0, 0),
            Ok(None)
        );
    }

    /* TWO-ELEMENT ARRAY */

    #[test]
    fn can_find_elements_in_two_element_array() {
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &0, 0, 1),
            Ok(Some(0))
        );
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &1, 0, 1),
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
                find_value(new_lookup(arr), t, 0, 1),
                Ok(None)
            );
        }
    }

    /* SINGLETON SUBSET OF TWO-ELEMENT ARRAY */

    #[test]
    fn can_find_element_in_singleton_subset_of_two_element_array() {
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &0, 0, 0),
            Ok(Some(0))
        );
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &1, 1, 1),
            Ok(Some(1))
        );
    }

    #[test]
    fn cannot_find_element_not_present_in_singleton_subset_of_two_element_array() {
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &0, 1, 1),
            Ok(None)
        );
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &1, 0, 0),
            Ok(None)
        );
    }

    /* LONGER ARRAYS */

    #[test]
    fn can_find_element_in_three_element_array() {
        let arr = &[0, 1, 2];
        for v in arr {
            assert_matches!(
                find_value(new_lookup(arr), v, 0, 2),
                Ok(Some(r)) if r == *v
            );
        }
    }

    /* USING SNAPPING */

    #[test]
    fn can_find_element_in_four_element_array_using_any_snap() {
        let arr = [0, 1, 2, 3];
        for v in arr {
            assert_matches!(
                find_value(new_lookup(&arr), &v, 0, 3),
                Ok(Some(r)) if r == v
            );
            assert_matches!(
                find_value(new_lookup(&arr), &with_snap(v, Some(Snap::Downwards)), 0, 3),
                Ok(Some(r)) if r == v
            );
            assert_matches!(
                find_value(new_lookup(&arr), &with_snap(v, Some(Snap::Upwards)), 0, 3),
                Ok(Some(r)) if r == v
            );
        }
    }

    #[test]
    fn can_find_element_before_unmatched_target_using_backwards_snap() {
        assert_matches!(
            find_value(new_lookup(&[0, 2]), &with_snap(1, Some(Snap::Downwards)), 0, 1),
            Ok(Some(0))
        );
    }

    #[test]
    fn can_find_element_after_unmatched_target_using_forwards_snap() {
        assert_matches!(
            find_value(new_lookup(&[0, 2]), &with_snap(1, Some(Snap::Upwards)), 0, 1),
            Ok(Some(2))
        );
    }

    #[test]
    fn can_find_first_element_for_target_less_than_first_element_using_forwards_snap() {
        assert_matches!(
            find_value(new_lookup(&[1, 2]), &with_snap(0, Some(Snap::Upwards)), 0, 1),
            Ok(Some(1))
        );
    }

    #[test]
    fn can_find_last_element_for_target_greater_than_last_element_using_backwards_snap() {
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &with_snap(2, Some(Snap::Downwards)), 0, 1),
            Ok(Some(1))
        );
    }

    #[test]
    fn cannot_find_element_before_target_less_than_first_element_using_backwards_snap() {
        assert_matches!(
            find_value(new_lookup(&[1, 2]), &with_snap(0, Some(Snap::Downwards)), 0, 1),
            Ok(None)
        );
    }

    #[test]
    fn cannot_find_element_after_target_greater_than_last_element_using_forwards_snap() {
        assert_matches!(
            find_value(new_lookup(&[0, 1]), &with_snap(2, Some(Snap::Upwards)), 0, 1),
            Ok(None)
        );
    }
}
