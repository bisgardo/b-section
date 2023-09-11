use crate::find::{find, FindOrd, Element, CmpResult, FindResult};

struct FindOrdRange<'a, T> {
    lower: &'a dyn FindOrd<T>,
    upper: &'a dyn FindOrd<T>,
}

impl<T> FindOrd<T> for FindOrdRange<'_, T> {
    fn lt(&self, v: &T) -> CmpResult {
        self.upper.lt(v)
    }

    fn gt(&self, v: &T) -> CmpResult {
        self.lower.gt(v)
    }
}

pub fn find_range<T, E>(
    lookup: &impl Fn(i64) -> Result<T, E>,
    lower_target: &dyn FindOrd<T>,
    upper_target: &dyn FindOrd<T>,
    lower_idx: i64, // inclusive
    upper_idx: i64, // inclusive
) -> Result<(Option<Element<T>>, Option<Element<T>>), E> {
    // TODO: Passing the target's snap in this call may result in an element that's outside the limits of last_lower/upper_idx (if no actual matching element is found).
    //       Should we "erase" the snap to avoid this?
    let FindResult { element, last_lower_idx, last_upper_idx } = find(
        lookup,
        &FindOrdRange { lower: lower_target, upper: upper_target },
        lower_idx,
        upper_idx,
    )?;
    match element {
        None => Ok((None, None)), // no value found in range
        Some(Element { index, .. }) => {
            // TODO: If value isn't found, the element before/after last_lower/upper_idx still works if the appropriate inequality snaps out!
            let lower_res = find(
                lookup,
                lower_target,
                last_lower_idx,
                index,
            )?;
            let upper_res = find(
                lookup,
                upper_target,
                index,
                last_upper_idx,
            )?;
            Ok((lower_res.element, upper_res.element))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::helpers::*;
    use assert_matches::assert_matches;
    use crate::find::Snap; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    #[test]
    fn range() {
        let arr = &[0, 1, 4, 5, 7, 9, 10, 13, 17, 21];
        assert_matches!(
            find_range(&new_lookup(arr), &6, &13, 0, arr.len() as i64),
            Ok((None, Some(u))) if u.value == 13
        );
    }

    #[test]
    fn range_with_snap() {
        let arr = &[0, 1, 4, 5, 7, 9, 10, 13, 17, 21];
        assert_matches!(
            find_range(&new_lookup(arr), &with_snap(6, Some(Snap::Downwards)), &13, 0, arr.len() as i64),
            Ok((Some(l), Some(u))) if l.value == 5 && u.value == 13
        );
    }
}
