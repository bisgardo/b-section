use crate::find::{FindOrd, FindOrdering};

pub struct FindOrdCombineLower<T, E> {
    pub combined: Vec<Box<dyn FindOrd<T, E>>>,
    pub snap_downwards: bool,
    pub snap_upwards: bool,
}

// Note that we cannot correctly infer 'is_valid_res' from the combined targets in all cases:
// - In the "any" direction, we'll automatically end up picking the most restrictive target
//   as that'll be the only one left at the edge.
// - In the "all" direction, we cannot know from the information available in a single call
//   which target is the most restrictive one.
//   A possible workaround could be to check the next element with the opposite inequality.
//   Then you would get the "opposite" 'is_valid_res' of the most restrictive target(s)
//   which is (probably) the opposite of the one we were looking for.
//   Doing that seems like severe overkill though and you could also ask
//   what should even happen in cases like where you have two targets at the same place but with different snap...
impl<T: std::fmt::Debug, E> FindOrd<T, E> for FindOrdCombineLower<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut val_below_all_targets = true; // value is below if it is according to all targets
        let mut val_above_any_target = false; // value is above it it is according to any target
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

pub struct FindOrdCombineUpper<T, E> {
    // TODO Use slice? Lifetime vs Box?
    pub combined: Vec<Box<dyn FindOrd<T, E>>>,
    pub snap_downwards: bool,
    pub snap_upwards: bool,
}

impl<T: std::fmt::Debug, E> FindOrd<T, E> for FindOrdCombineUpper<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut val_below_any_target = false; // value is below if it is according to any target
        let mut val_above_all_targets = true; // value is above it it is according to all targets
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
