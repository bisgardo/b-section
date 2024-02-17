use crate::find::{FindOrd, FindOrdering};

/// Implementation of [`FindOrd`] that allows a value to be effectively compared
/// against the "lowermost" target in a given set of targets.
///
/// That is, the comparison result is defined as "lesser" if the value is below *all* targets
/// and "greater" if the value is above *any* target.
///
/// This is the opposite of [FindOrdCombineUpper] which compares against the "uppermost" target.
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
///
/// This is the opposite of [FindOrdCombineLower] which compares against the "lowermost" target.
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
