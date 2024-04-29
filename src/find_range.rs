use crate::find::{find, Element, FindOrd, FindOrdering, FindResult};
use std::cmp::{max, min};

struct FindOrdRange<'a, T, E> {
    lower: &'a dyn FindOrd<T, E>,
    upper: &'a dyn FindOrd<T, E>,
}

impl<T, E> FindOrd<T, E> for FindOrdRange<'_, T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        Ok(
            if let FindOrdering::ValAboveTarget { .. } = self.upper.cmp(t)? {
                FindOrdering::ValAboveTarget { is_valid_res: false } // erasing 'is_valid_res'
            } else if let FindOrdering::ValBelowTarget { .. } = self.lower.cmp(t)? {
                FindOrdering::ValBelowTarget { is_valid_res: false } // erasing 'is_valid_res'
            } else {
                FindOrdering::ValMatchesTarget
            }
        )
    }
}

pub fn find_range<T, E>(
    lookup: &impl Fn(i64) -> Result<T, E>,
    lower_target: &dyn FindOrd<T, E>,
    upper_target: &dyn FindOrd<T, E>,
    lower_idx: i64, // inclusive
    upper_idx: i64, // inclusive
) -> Result<(Option<Element<T>>, Option<Element<T>>), E> {
    // println!("find in range {}-{}", lower_idx, upper_idx);
    let FindResult { element, last_lower_idx, last_upper_idx } = find(
        lookup,
        &FindOrdRange { lower: lower_target, upper: upper_target },
        lower_idx,
        upper_idx,
    )?;
    let (lower_upper_idx, upper_lower_idx) = match element {
        // Element was not found: the lower and upper limits will be on each side (reversed) of the index where the element would have been.
        // It's possible that "snap out" values will be found up to/down from this index.
        // TODO: If we didn't erase 'is_valid_res' and kept both last upper- and lower valid result,
        //       then we'd have the final result right away in this case?!?
        None => (last_upper_idx, last_lower_idx),
        // TODO: Isn't '(idx, idx-1)' sufficient? The range is extended below anyway and we found the value without snapping.
        Some(Element { idx, .. }) => (idx, idx),
    };
    // Possible optimization: If we can determine that the targets aren't using outwards snapping,
    // then the min/max expansion don't add anything.
    // We could probably also spare one of the following 'find's entirely if the snapping wasn't erased in 'FindOrdRange'
    // and we recorded the snap capture of the element (if it was snap downwards then the value would equal 'lower_res' and vice versa).
    // It isn't clear what the implications of using 'idx' from such a value is through.
    // But we can always detect "not found" as 'last_lower_idx > last_upper_idx'.

    // Search for lower target in range ['last_lower_idx'; 'idx']
    // (expanded a bit to ensure that "snap down" values are found.)
    let lower_res = find(
        lookup,
        lower_target,
        max(last_lower_idx - 1, lower_idx), // expand down by 1 (limited by 'lower_idx') to ensure that any "snap down" value is found
        lower_upper_idx, // if no element was found in range, this equals 'last_upper_idx'
    )?;

    // Search for lower target in range ['idx'; 'last_upper_idx']
    // (expanded a bit to ensure that "snap up" values are found).
    let upper_res = find(
        lookup,
        upper_target,
        upper_lower_idx, // if no element was found in range, then this equals 'last_lower_idx'
        min(last_upper_idx + 1, upper_idx), // expand up by 1 (limited by 'upper_idx') to ensure that any "snap up" value is found
    )?;
    Ok((lower_res.element, upper_res.element))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::helpers::*;
    use assert_matches::assert_matches; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    fn all_snap_variants<E>(v: i64) -> Vec<Box<dyn FindOrd<i64, E>>> {
        vec![
            Box::new(v),
            Box::new(with_snap(v, Snap::Downwards)),
            Box::new(with_snap(v, Snap::Upwards)),
        ]
    }

    #[test]
    fn full_range() {
        let arr = &[0, 2, 4];
        for lt in all_snap_variants(0) {
            for ut in all_snap_variants(4) {
                assert_matches!(
                    find_range(&new_lookup(arr), lt.as_ref(), ut.as_ref(), 0, arr.len() as i64 - 1),
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
        let arr = [0, 2, 4];
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
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &1, &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &1, &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Upwards), &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 2
        );
    }

    #[test]
    fn below_element() {
        let arr = &[0];
        assert_matches!(
            find_range(&new_lookup(arr), &-1, &-1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(-1, Snap::Upwards), &-1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &-1, &with_snap(-1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(-1, Snap::Upwards), &with_snap(-1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 0
        );
    }

    #[test]
    fn above_element() {
        let arr = &[0];
        assert_matches!(
            find_range(&new_lookup(arr), &1, &1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &1, 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &1, &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(1, Snap::Downwards), &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 0
        );
    }

    /* REVERSED RANGES */

    #[test]
    fn reversed_targets_subrange() {
        let arr = &[0, 1, 2, 3, 4];
        // The results of queries where the "lower" target is above the "higher" one are undefined,
        // so the following tests capture the current behavior without necessarily indicating that the behavior is intended:
        // The results are not too far off, but still pretty weird.
        // It would be perfectly fine if a future change broke them (they probably should all return empty results).
        // Including them here ensures that such a change will not happen accidentally.
        assert_matches!(
            find_range(&new_lookup(arr), &3, &1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &3, &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &3, &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Downwards), &1, 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 1
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Downwards), &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 1
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Downwards), &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 1 && u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Upwards), &1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Upwards), &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Upwards), &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 2
        );
    }

    #[test]
    fn reversed_targets_between() {
        let arr = &[0, 2, 4];
        // The results of queries where the "lower" target is above the "higher" one are undefined,
        // so the following tests capture the current behavior without necessarily indicating that the behavior is intended:
        // The results are not too far off, but still pretty weird.
        // It would be perfectly fine if a future change broke them (they probably should all return empty results).
        // Including them here ensures that such a change will not happen accidentally.
        assert_matches!(
            find_range(&new_lookup(arr), &3, &1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &3, &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &3, &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Downwards), &1, 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Downwards), &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), None)) if l.val == 0
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Downwards), &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((Some(l), Some(u))) if l.val == 0 && u.val == 2
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Upwards), &1, 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Upwards), &with_snap(1, Snap::Downwards), 0, arr.len() as i64 - 1),
            Ok((None, None))
        );
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(3, Snap::Upwards), &with_snap(1, Snap::Upwards), 0, arr.len() as i64 - 1),
            Ok((None, Some(u))) if u.val == 2
        );
    }

    // TODO: Add test where lower/upper targets are swapped. They should not find any results...
}
