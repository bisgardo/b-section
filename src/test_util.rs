#[cfg(test)]
pub mod helpers {
    use crate::find::{FindOrd, FindOrdering};

    #[derive(Clone)]
    pub enum Snap {
        Downwards,
        Upwards,
    }

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

    impl<T: PartialOrd, E> FindOrd<T, E> for SnappingTarget<T> {
        fn cmp(&self, t: &T) -> Result<FindOrdering, E> {
            Ok(
                if &self.value < t {
                    FindOrdering::ValAboveTarget { is_valid_res: matches!(self.snap, Some(Snap::Upwards)) }
                } else if &self.value > t {
                    FindOrdering::ValBelowTarget { is_valid_res: matches!(self.snap, Some(Snap::Downwards)) }
                } else {
                    FindOrdering::ValMatchesTarget
                }
            )
        }
    }

    pub fn with_snap<T>(value: T, snap: Snap) -> SnappingTarget<T> {
        SnappingTarget { value, snap: Some(snap) }
    }
}
