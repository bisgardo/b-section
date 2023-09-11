#[cfg(test)]
pub mod helpers {
    use crate::find::{FindOrd, CmpResult, Snap};

    pub fn new_lookup<T: Clone>(arr: &[T]) -> impl Fn(i64) -> Result<T, String> + '_ {
        |idx| {
            if idx < 0 {
                return Err(format!("negative index {}", idx));
            }
            let idx = idx as usize;
            if idx >= arr.len() {
                return Err(format!("index {} out of bounds", idx));
            }
            Ok(arr[idx].clone())
        }
    }

    pub struct SnappingTarget<T> {
        value: T,
        snap: Option<Snap>,
    }

    impl<T: PartialOrd> FindOrd<T> for SnappingTarget<T> {
        fn lt(&self, t: &T) -> CmpResult {
            if &self.value < t {
                CmpResult::True { keep: matches!(self.snap, Some(Snap::Upwards)) }
            } else {
                CmpResult::False
            }
        }

        fn gt(&self, t: &T) -> CmpResult {
            if &self.value > t {
                CmpResult::True { keep: matches!(self.snap, Some(Snap::Downwards)) }
            } else {
                CmpResult::False
            }
        }
    }

    pub fn with_snap<T>(value: T, snap: Option<Snap>) -> SnappingTarget<T> {
        SnappingTarget { value, snap }
    }
}
