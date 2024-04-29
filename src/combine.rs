use crate::find::{FindOrd, FindOrdering};

/// Implementation of [`FindOrd`] that allows a value to be effectively compared
/// against the "lowermost" target in a given set of targets.
///
/// That is, the comparison result is defined as "lesser" if the value is below *all* targets
/// and "greater" if the value is above *any* target.
/// This is the opposite of [FindOrdCombineUpper] which compares against the "uppermost" target.
///
/// This corresponds to logical OR ("any") of the target conditions when used as lower limit ("from")
/// and AND ("all") when used as the upper limit ("to").
///
/// The `is_valid_res` fields of the individual comparison results are ignored
/// because combining them correctly in all cases is non-trivial at best.
/// Instead, the returned value if `is_valid_res` is
/// - `snap_downwards` if the result is [`FindOrdering::ValBelowTarget`]
/// - `snap_upwards` if the result is [`FindOrdering::ValAboveTarget`].
pub struct FindOrdCombineLower<T, E> {
    // TODO Use slice? Lifetime vs Box?
    pub combined: Vec<Box<dyn FindOrd<T, E>>>,
    pub snap_downwards: bool,
    pub snap_upwards: bool,
}

impl<T, E> FindOrd<T, E> for FindOrdCombineLower<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut val_below_all_targets = true; // value is below if all targets say so
        let mut val_above_any_target = false; // value is above if any target says so
        for f in self.combined.iter() {
            match f.cmp(t)? {
                FindOrdering::ValBelowTarget { .. } => {}
                FindOrdering::ValAboveTarget { .. } => {
                    val_below_all_targets = false;
                    val_above_any_target = true;
                }
                FindOrdering::ValMatchesTarget => {
                    val_below_all_targets = false;
                }
            }
        }
        Ok(
            if val_below_all_targets {
                //println!("cmp {:?} = val below target (valid={})", t, self.snap_downwards);
                FindOrdering::ValBelowTarget { is_valid_res: self.snap_downwards }
            } else if val_above_any_target {
                //println!("cmp {:?} = val above target (valid={})", t, self.snap_upwards);
                FindOrdering::ValAboveTarget { is_valid_res: self.snap_upwards }
            } else {
                //println!("cmp {:?} = match", t);
                FindOrdering::ValMatchesTarget
            }
        )
    }
}

/// Implementation of [`FindOrd`] that allows a value to be effectively compared
/// against the "uppermost" target in a given set of targets.
///
/// That is, the comparison result is defined as "lesser" if the value is below *any* target
/// and "greater" if the value is above *all* targets.
/// This is the opposite of [FindOrdCombineLower] which compares against the "lowermost" target.
///
/// This corresponds to logical AND ("all") of the target conditions when used as lower limit ("from")
/// and OR ("any") when used as the upper limit ("to").
///
/// The `is_valid_res` fields of the individual comparison results are ignored
/// because combining them correctly in all cases is non-trivial at best.
/// Instead, the returned value if `is_valid_res` is
/// - `snap_downwards` if the result is [`FindOrdering::ValBelowTarget`]
/// - `snap_upwards` if the result is [`FindOrdering::ValAboveTarget`].
pub struct FindOrdCombineUpper<T, E> {
    pub combined: Vec<Box<dyn FindOrd<T, E>>>,
    pub snap_downwards: bool,
    pub snap_upwards: bool,
}

impl<T, E> FindOrd<T, E> for FindOrdCombineUpper<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut val_below_any_target = false; // value is below if any target says so
        let mut val_above_all_targets = true; // value is above if all targets say so
        for f in self.combined.iter() {
            match f.cmp(t)? {
                FindOrdering::ValBelowTarget { .. } => {
                    val_below_any_target = true;
                    val_above_all_targets = false;
                }
                FindOrdering::ValAboveTarget { .. } => {}
                FindOrdering::ValMatchesTarget => {
                    val_above_all_targets = false;
                }
            }
        }
        Ok(
            if val_above_all_targets {
                //println!("cmp {:?} = val above target (valid={})", t, self.snap_upwards);
                FindOrdering::ValAboveTarget { is_valid_res: self.snap_upwards }
            } else if val_below_any_target {
                //println!("cmp {:?} = val below target (valid={})", t, self.snap_downwards);
                FindOrdering::ValBelowTarget { is_valid_res: self.snap_downwards }
            } else {
                //println!("cmp {:?} = match", t);
                FindOrdering::ValMatchesTarget
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::helpers::*;
    use assert_matches::assert_matches; // use stdlib version once it's stable (https://github.com/rust-lang/rust/issues/82775)

    #[derive(Clone, Debug, PartialEq)]
    struct Data {
        pub a: f64,
        pub b: f64,
    }

    enum Field {
        A,
        B,
    }

    struct Target {
        pub field: Field,
        pub val: f64,
    }

    impl FindOrd<Data, String> for Target {
        fn cmp(&self, t: &Data) -> Result<FindOrdering, String> {
            let data_val = match self.field {
                Field::A => t.a,
                Field::B => t.b,
            };
            Ok(
                if self.val < data_val {
                    FindOrdering::ValAboveTarget { is_valid_res: true }
                } else if self.val > data_val {
                    FindOrdering::ValBelowTarget { is_valid_res: false }
                } else {
                    FindOrdering::ValMatchesTarget
                }
            )
        }
    }

    // TODO: Test much more thoroughly.

    #[test]
    fn can_find_element_using_targets_combined_into_or_of_and() {
        let and1 = FindOrdCombineUpper {
            combined: vec![
                Box::new(Target { field: Field::A, val: 1.0 }), // a >= 1.0
                Box::new(Target { field: Field::B, val: 2.0 }), // b >= 2.0
            ],
            snap_downwards: false, // ignored
            snap_upwards: true,    // ignored
        };

        let and2 = FindOrdCombineUpper {
            combined: vec![
                Box::new(Target { field: Field::A, val: 0.1 }), // a >= 0.1
                Box::new(Target { field: Field::B, val: 4.0 }), // b >= 4.1
            ],
            snap_downwards: false, // ignored
            snap_upwards: true,    // ignored
        };

        let and1_or_and2 = FindOrdCombineLower {
            combined: vec![Box::new(and1), Box::new(and2)],
            snap_downwards: false,
            snap_upwards: true,
        };

        let data = [
            Data { a: 0.0, b: 1.0 },
            Data { a: 1.0, b: 2.0 },
            Data { a: 2.0, b: 4.0 },
        ];
        let r = crate::find::find(
            &new_lookup(&data),
            &and1_or_and2,
            0,
            data.len() as i64 - 1,
        ).unwrap();

        assert_matches!(
            r.element,
            Some(crate::find::Element {val: d, idx: 1}) if d == Data {a: 1.0 , b: 2.0 }
        );
    }

    #[test]
    fn can_find_element_using_targets_combined_using_and_of_or() {
        let or1 = FindOrdCombineLower {
            combined: vec![
                Box::new(Target { field: Field::A, val: 1.0 }), // a >= 1.0
                Box::new(Target { field: Field::B, val: 2.0 }), // b >= 2.0
            ],
            snap_downwards: false, // ignored
            snap_upwards: true,    // ignored
        };

        let or2 = FindOrdCombineLower {
            combined: vec![
                Box::new(Target { field: Field::A, val: 0.1 }), // a >= 0.1
                Box::new(Target { field: Field::B, val: 4.0 }), // b >= 4.1
            ],
            snap_downwards: false, // ignored
            snap_upwards: true,    // ignored
        };

        let or1_and_or2 = FindOrdCombineUpper {
            combined: vec![Box::new(or1), Box::new(or2)],
            snap_downwards: false,
            snap_upwards: true,
        };

        let data = [
            Data { a: 0.0, b: 1.0 },
            Data { a: 1.0, b: 2.0 },
            Data { a: 2.0, b: 4.0 },
        ];
        let r = crate::find::find(
            &new_lookup(&data),
            &or1_and_or2,
            0,
            data.len() as i64 - 1,
        ).unwrap();

        assert_matches!(
            r.element,
            Some(crate::find::Element {val: d, idx: 1}) if d == Data {a: 1.0 , b: 2.0 }
        );
    }
}
