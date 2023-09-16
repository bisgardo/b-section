use std::collections::HashMap;
use b_section::find::{FindOrdering, FindOrd, Snap};
use crate::pair::{Op, Pair};

pub type Data = HashMap<String, f64>;

pub struct DataTarget {
    pub name: String,
    pub val: f64,
    pub snap: Option<Snap>,
}

impl<E> FindOrd<Data, E> for DataTarget {
    fn cmp(&self, t: &Data) -> Result<FindOrdering, E> {
        let val = t[&self.name];
        Ok(
            if self.val < val {
                FindOrdering::ValAboveTarget { is_valid_res: matches!(self.snap, Some(Snap::Upwards)) }
            } else if self.val > val {
                FindOrdering::ValBelowTarget { is_valid_res: matches!(self.snap, Some(Snap::Downwards)) }
            } else {
                FindOrdering::ValMatchesTarget
            }
        )
    }
}

impl DataTarget {
    pub fn from_pair<E>(p: Pair, t: Target) -> Result<Box<dyn FindOrd<Data, E>>, anyhow::Error> {
        let name = p.name;
        let val = p.value.parse()?;
        let snap = match (t, p.op) {
            (Target::Lower, Op::Tilde) => Snap::Downwards,
            (Target::Lower, Op::Equals) => Snap::Upwards,
            (Target::Upper, Op::Tilde) => Snap::Upwards,
            (Target::Upper, Op::Equals) => Snap::Downwards,
        };
        Ok(Box::new(DataTarget { name, val, snap: Some(snap) }))
    }
}

#[derive(Clone)]
pub enum Target {
    Lower,
    Upper,
}
