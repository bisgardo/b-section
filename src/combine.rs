use crate::find::{CmpResult, FindOrd};

/// [FindOrd] that "matches" (i.e. inequality is [CmpResult::False]) if ANY of the combined [FindOrd]s say so.
// TODO: Create variant with "all" semantics.
struct FindOrdCombinerAny<T> {
    combined: Vec<Box<dyn FindOrd<T>>>, // TODO use slice? And lifetime vs Box?
}

impl<T> FindOrd<T> for FindOrdCombinerAny<T> {
    fn lt(&self, t: &T) -> CmpResult {
        // Target is less than the value if it is according to all targets (i.e. it isn't if *any* of the targets say so).
        // Default to keeping the candidate value but don't if any of the targets say we shouldn't.
        let mut all_keep = true;
        for f in self.combined.iter() {
            match f.lt(t) {
                CmpResult::False => return CmpResult::False,
                CmpResult::True { keep } => {
                    all_keep &= keep
                }
            }
        }
        CmpResult::True { keep: all_keep }
    }

    fn gt(&self, t: &T) -> CmpResult {
        // Target is greater than the value if it is according to all targets (i.e. it isn't if *any* of the targets say so).
        // Default to keeping the candidate value but don't if any of the targets say we shouldn't.
        let mut all_keep = true;
        for f in self.combined.iter() {
            match f.gt(t) {
                CmpResult::False => return CmpResult::False,
                CmpResult::True { keep } =>
                    all_keep &= keep
            }
        }
        CmpResult::True { keep: all_keep }
    }
}
