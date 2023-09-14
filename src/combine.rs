use crate::find::{FindOrd, FindOrdering};

/// [FindOrd] that "matches" (i.e. inequality is [CmpResult::False]) if ANY of the combined [FindOrd]s say so.
// TODO: Create variant with "all" semantics.
struct FindOrdCombinerAny<T, E> {
    combined: Vec<Box<dyn FindOrd<T, E>>>, // TODO use slice? And lifetime vs Box?
}

impl<T, E> FindOrd<T, E> for FindOrdCombinerAny<T, E> {
    fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
        let mut all_val_below_target = true;
        let mut all_val_above_target = true;
        let mut all_valid_res = true;
        for f in self.combined.iter() {
            match f.cmp(t)? {
                FindOrdering::ValBelowTarget { is_valid_res } => {
                    all_val_above_target = false;
                    all_valid_res &= is_valid_res;
                }
                FindOrdering::ValAboveTarget { is_valid_res } => {
                    all_val_below_target = false;
                    all_valid_res &= is_valid_res;
                }
                FindOrdering::ValMatchesTarget => {
                    all_val_below_target = false;
                    all_val_above_target = false;
                }
            }
        }
        Ok(
            match (all_val_below_target, all_val_above_target) {
                (true, false) => FindOrdering::ValBelowTarget { is_valid_res: all_valid_res },
                (false, true) => FindOrdering::ValAboveTarget { is_valid_res: all_valid_res },
                _ => FindOrdering::ValMatchesTarget,
            }
        )
    }
}
