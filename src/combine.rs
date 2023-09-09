use crate::find::{CmpResult, FindOrd, Snap};

/// [FindOrd] that "matches" (i.e. inequality is [CmpResult::False]) if ANY of the combined [FindOrd]s say so.
// TODO: Create variant with "all" semantics.
struct FindOrdCombinerAny<T> {
    combined: Vec<Box<dyn FindOrd<T>>>, // TODO use slice? And lifetime vs Box?
}

impl<T> FindOrd<T> for FindOrdCombinerAny<T> {
    fn lt(&self, t: &T) -> CmpResult {
        // Target is less than the value if all of the targets say so (i.e. it isn't if *any* of the targets say so).
        // Default to snapping upwards (outwards) but snap downwards if any of the targets say so.
        let mut snap = Snap::Upwards;
        for f in self.combined.iter() {
            match f.lt(t) {
                CmpResult::False => return CmpResult::False,
                CmpResult::True(s) =>
                    if let Some(Snap::Downwards) = s {
                        snap = Snap::Downwards;
                    }
            }
        }
        CmpResult::True(Some(snap))
    }

    fn gt(&self, t: &T) -> CmpResult {
        // Target is greater than the value if all of the targets say so (i.e. it isn't if *any of the targets say so).
        // Default to snapping downwards (outwards) but snap upwards if any of the targets say so.
        let mut snap = Snap::Downwards;
        for f in self.combined.iter() {
            match f.gt(t) {
                CmpResult::False => return CmpResult::False,
                CmpResult::True(s) =>
                    if let Some(Snap::Upwards) = s {
                        snap = Snap::Upwards;
                    }
            }
        }
        CmpResult::True(Some(snap))
    }
}
