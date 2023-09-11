use anyhow::Result;
use std::collections::HashMap;
use b_section::find::{CmpResult, FindOrd, Snap};
use crate::pair::{Op, Pair};

pub type Data = HashMap<String, f64>;

pub struct DataTarget {
    pub name: String,
    pub value: f64,
    pub snap: Option<Snap>,
}

impl FindOrd<Data> for DataTarget {
    fn lt(&self, t: &Data) -> CmpResult {
        if self.value < t[&self.name] {
            CmpResult::True { keep: matches!(self.snap, Some(Snap::Upwards)) }
        } else {
            CmpResult::False
        }
    }

    fn gt(&self, t: &Data) -> CmpResult {
        if self.value > t[&self.name] {
            CmpResult::True { keep: matches!(self.snap, Some(Snap::Downwards)) }
        } else {
            CmpResult::False
        }
    }
}

impl DataTarget {
    pub fn from_pair(p: Pair, t: Target) -> Result<Box<dyn FindOrd<Data>>> {
        let name = p.name;
        let value = p.value.parse()?;
        let snap = match (t, p.op) {
            (Target::Lower, Op::Tilde) => Snap::Downwards,
            (Target::Lower, Op::Equals) => Snap::Upwards,
            (Target::Upper, Op::Tilde) => Snap::Upwards,
            (Target::Upper, Op::Equals) => Snap::Downwards,
        };
        Ok(Box::new(DataTarget { name, value, snap: Some(snap) }))
    }
}

pub enum Target {
    Lower,
    Upper,
}
