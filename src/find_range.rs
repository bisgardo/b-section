use crate::find::{find, FindOrd, Element, Snap};

struct FindOrdRange<'a, T> {
    lower: &'a dyn FindOrd<T>,
    upper: &'a dyn FindOrd<T>,
}

impl<T> FindOrd<T> for FindOrdRange<'_, T> {
    fn lt(&self, v: &T) -> bool {
        self.upper.lt(v)
    }

    fn gt(&self, v: &T) -> bool {
        self.lower.gt(v)
    }
}

pub fn find_range<T, E>(
    lookup: &impl Fn(i64) -> Result<T, E>,
    lower_target: &impl FindOrd<T>,
    upper_target: &impl FindOrd<T>,
    lower_idx: i64, // inclusive
    upper_idx: i64, // inclusive
) -> Result<(Option<Element<T>>, Option<Element<T>>), E> {
    // TODO Move snapping to 'FindOrd'.
    let range_res = find(
        lookup,
        &FindOrdRange { lower: lower_target, upper: upper_target },
        lower_idx,
        upper_idx,
        None,
    )?;
    // TODO Bail if no value within range was found.
    let lower_res = find(
        lookup,
        lower_target,
        range_res.lower_bound,
        match range_res.element {
            None => range_res.upper_bound,
            Some(Element { index, .. }) => index,
        },
        Some(Snap::Upwards),
    )?;
    let upper_res = find(
        lookup,
        upper_target,
        match range_res.element {
            None => range_res.lower_bound,
            Some(Element { index, .. }) => index,
        },
        range_res.upper_bound,
        Some(Snap::Downwards),
    )?;
    Ok((lower_res.element, upper_res.element))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::helpers::new_lookup;
    use assert_matches::assert_matches; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    #[test]
    fn range() {
        let arr = &[0, 1, 4, 5, 7, 9, 10, 13, 17, 21];
        assert_matches!(
            find_range(&new_lookup(arr), &6, &13, 0, arr.len() as i64),
            Ok((Some(l), Some(u))) if l.value == 7 && u.value == 13
        );
    }
}
