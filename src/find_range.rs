use std::cmp::{max, min};
use crate::find::{find, FindOrd, Element, CmpResult, FindResult};

struct FindOrdRange<'a, T> {
    lower: &'a dyn FindOrd<T>,
    upper: &'a dyn FindOrd<T>,
}

impl<T> FindOrd<T> for FindOrdRange<'_, T> {
    fn lt(&self, v: &T) -> CmpResult {
        self.upper.lt(v).no_keep()
    }

    fn gt(&self, v: &T) -> CmpResult {
        self.lower.gt(v).no_keep()
    }
}

pub fn find_range<T, E>(
    lookup: &impl Fn(i64) -> Result<T, E>,
    lower_target: &dyn FindOrd<T>,
    upper_target: &dyn FindOrd<T>,
    lower_idx: i64, // inclusive
    upper_idx: i64, // inclusive
) -> Result<(Option<Element<T>>, Option<Element<T>>), E> {
    let FindResult { element, last_lower_idx, last_upper_idx } = find(
        lookup,
        &FindOrdRange { lower: lower_target, upper: upper_target },
        lower_idx,
        upper_idx,
    )?;
    let (lower_upper_idx, upper_lower_idx) = match element {
        None => (last_lower_idx, last_upper_idx),
        Some(Element { idx, .. }) => (idx, idx),
    };
    // Possible optimization: If we can determine that the targets aren't using outwards snapping,
    // then the min/max expansion don't add anything.
    // We could probably also spare one of the following 'find's entirely if the snapping wasn't erased in 'FindOrdRange'
    // and we recorded the snap capture of the element (if it was snap downwards then the value would equal 'lower_res' and vice versa).
    // It isn't clear what the implications of using 'idx' from such a value is through.
    // But we can always detect "not found" as 'last_lower_idx > last_upper_idx'.
    let lower_res = find(
        lookup,
        lower_target,
        max(last_lower_idx - 1, lower_idx), // necessary to ensure that we find any "snap down" value
        lower_upper_idx,
    )?;
    let upper_res = find(
        lookup,
        upper_target,
        upper_lower_idx,
        min(last_upper_idx + 1, upper_idx), // necessary to ensure that we find any "snap up" value
    )?;
    Ok((lower_res.element, upper_res.element))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::helpers::*;
    use assert_matches::assert_matches;
    use crate::find::Snap; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    fn all_snap_variants(v: i64) -> Vec<Box<dyn FindOrd<i64>>> {
        vec![
            Box::new(v),
            Box::new(with_snap(v, Snap::Downwards)),
            Box::new(with_snap(v, Snap::Upwards)),
        ]
    }

    #[test]
    fn full_range() {
        let arr = [0, 2, 4];
        for lt in all_snap_variants(0) {
            for ut in all_snap_variants(4) {
                assert_matches!(
                    find_range(&new_lookup(&arr), lt.as_ref(), ut.as_ref(), 0, arr.len() as i64 - 1),
                    Ok((Some(l), Some(u))) if l.val == 0 && u.val == 4
                );
            }
        }
    }

    #[test]
    fn matching_sub_range() {
        let arr = &[0, 2, 4];
        for lt in all_snap_variants(0) {
            for ut in all_snap_variants(2) {
                assert_matches!(
                    find_range(&new_lookup(arr), lt.as_ref(), ut.as_ref(), 0, arr.len() as i64 - 1),
                    Ok((Some(l), Some(u))) if l.val == 0 && u.val == 2
                );
            }
        }
        for lt in all_snap_variants(2) {
            for ut in all_snap_variants(4) {
                assert_matches!(
                    find_range(&new_lookup(arr), lt.as_ref(), ut.as_ref(), 0, arr.len() as i64 - 1),
                    Ok((Some(l), Some(u))) if l.val == 2 && u.val == 4
                );
            }
        }
    }

    #[test]
    fn matching_single_element() {
        let arr = [0i64, 2, 4];
        for v in arr {
            for lt in all_snap_variants(v) {
                for ut in all_snap_variants(v) {
                    assert_matches!(
                    find_range(&new_lookup(&arr), lt.as_ref(), ut.as_ref(), 0, arr.len() as i64 - 1),
                    Ok((Some(l), Some(u))) if l.val == v && u.val == v
                );
                }
            }
        }
    }

    #[test]
    fn snap_lower() {
        let arr = &[0, 2, 4];
        assert_matches!(
            find_range(&new_lookup(arr), &1, &4, 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 4
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &4, 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 4
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Upwards), &4, 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 2 && u.val == 4
        );
    }

    #[test]
    fn snap_upper() {
        let arr = &[0, 2, 4];
        assert_matches!(
            find_range(&new_lookup(arr), &0, &3, 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &0, &with_snap(3, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &0, &with_snap(3, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 4
        );
    }

    #[test]
    fn snap_lower_and_upper() {
        let arr = &[0, 2, 4];
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &with_snap(3, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &with_snap(3, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 4
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Upwards), &with_snap(3, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 2 && u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Upwards), &with_snap(3, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 2 && u.val == 4
        );
    }

    #[test]
    fn lower_bound_from_range_match_is_above_downwards_snap_value() {
        // Array is chosen such that when looking for lower value, the lower bound is above the downwards snap.
        // If this bound is used directly, then the snap value won't be found.
        let arr = &[-1, 0, 2, 4];
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &4, 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 4
        );
    }

    #[test]
    fn upper_bound_from_range_match_is_below_upwards_snap_value() {
        // Array is chosen such that when looking for upper value, the upper bound is below the upwards snap.
        // If this bound is used directly, then the snap value won't be found.
        let arr = &[0, 2, 4, 5];
        assert_matches!(
            find_range(&new_lookup(arr), &0, &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 2
        );
    }

    #[test]
    fn between_elements() {
        let arr = &[0, 2];
        assert_matches!(
            find_range(&new_lookup(arr), &1, &1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &1, 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Upwards), &1, 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &1, &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &1, &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 2
        );
    }
}
