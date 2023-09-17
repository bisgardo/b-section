use crate::find::{FindOrd, FindOrdering};

pub struct FindOrdCombineLower<T, E> {
    pub combined: Vec<Box<dyn FindOrd<T, E>>>, // TODO Use slice? Lifetime vs Box?
}

impl<T: std::fmt::Debug, E> FindOrd<T, E> for FindOrdCombineLower<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut val_below_all_targets = true; // value is below if it is according to all targets
        let mut val_above_any_target = false; // value is above it it is according to any target
        let mut valid_below = false;
        let mut valid_above = false;
        for f in self.combined.iter() {
            match f.cmp(t)? {
                FindOrdering::ValBelowTarget { is_valid_res } => {
                    valid_below |= is_valid_res;
                }
                FindOrdering::ValAboveTarget { is_valid_res } => {
                    val_below_all_targets = false;
                    val_above_any_target = true;
                    valid_above |= is_valid_res;
                }
                FindOrdering::ValMatchesTarget => {
                    val_below_all_targets = false;
                }
            }
        }
        Ok(
            if val_below_all_targets {
                println!("cmp {:?} = val below target (valid={})", t, valid_below);
                FindOrdering::ValBelowTarget { is_valid_res: valid_below }
            } else if val_above_any_target {
                println!("cmp {:?} = val above target (valid={})", t, valid_above);
                FindOrdering::ValAboveTarget { is_valid_res: valid_above }
            } else {
                println!("cmp {:?} = match", t);
                FindOrdering::ValMatchesTarget
            }
        )
    }
}

pub struct FindOrdCombineUpper<T, E> {
    pub combined: Vec<Box<dyn FindOrd<T, E>>>, // TODO Use slice? Lifetime vs Box?
}

impl<T: std::fmt::Debug, E> FindOrd<T, E> for FindOrdCombineUpper<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut val_below_any_target = false; // value is below if it is according to any target
        let mut val_above_all_targets = true; // value is above it it is according to all targets
        let mut valid_below = false;
        let mut valid_above = false;
        for f in self.combined.iter() {
            match f.cmp(t)? {
                FindOrdering::ValBelowTarget { is_valid_res } => {
                    val_below_any_target = true;
                    val_above_all_targets = false;
                    valid_below |= is_valid_res;
                }
                FindOrdering::ValAboveTarget { is_valid_res } => {
                    valid_above |= is_valid_res;
                }
                FindOrdering::ValMatchesTarget => {
                    val_above_all_targets = false;
                }
            }
        }
        Ok(
            if val_above_all_targets {
                println!("cmp {:?} = val above target (valid={})", t, valid_above);
                FindOrdering::ValAboveTarget { is_valid_res: valid_above }
            } else if val_below_any_target {
                println!("cmp {:?} = val below target (valid={})", t, valid_below);
                FindOrdering::ValBelowTarget { is_valid_res: valid_below }
            } else {
                println!("cmp {:?} = match", t);
                FindOrdering::ValMatchesTarget
            }
        )
    }
}
