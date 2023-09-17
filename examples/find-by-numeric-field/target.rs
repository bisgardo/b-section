use std::collections::HashMap;
use b_section::find::{FindOrdering, FindOrd};
use crate::pair::{Op, Pair};

pub type Data = HashMap<String, f64>;

#[derive(Clone)]
pub struct DataTarget {
    pub name: String,
    pub val: f64,
    pub snap_downwards: bool,
    pub snap_upwards: bool,
}

impl<E> FindOrd<Data, E> for DataTarget {
    fn cmp(&self, t: &Data) -> Result<FindOrdering, E> {
        let val = t[&self.name];
        Ok(
            if self.val < val {
                FindOrdering::ValAboveTarget { is_valid_res: self.snap_upwards }
            } else if self.val > val {
                FindOrdering::ValBelowTarget { is_valid_res: self.snap_downwards }
            } else {
                FindOrdering::ValMatchesTarget
            }
        )
    }
}

impl DataTarget {
    pub fn from_pair<E>(p: Pair, t: Target) -> Result<Box<DataTarget>, anyhow::Error> {
        let name = p.name;
        let val = p.value.parse()?;
        let snap_out = match p.op {
            Op::Equals => false,
            Op::Tilde => true,
        };
        let snap_upwards = match t {
            Target::Lower => !snap_out,
            Target::Upper => snap_out,
        };
        let snap_downwards = !snap_upwards;
        Ok(Box::new(DataTarget { name, val, snap_downwards, snap_upwards }))
    }
}

#[derive(Clone)]
pub enum Target {
    Lower,
    Upper,
}
