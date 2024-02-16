use crate::pair::{Op, Pair};
use anyhow::{anyhow, Error};
use b_section::find::{FindOrd, FindOrdering};
use std::collections::HashMap;

pub type Data = HashMap<String, f64>;

fn sorted_items<K: Ord, V>(m: &HashMap<K, V>) -> Vec<(&K, &V)> {
    let mut res: Vec<_> = m.iter().collect();
    res.sort_by(|&(l, _), &(r, _)| l.cmp(r));
    res
}

pub fn data_to_string(d: Data) -> String {
    sorted_items(&d)
        .iter()
        .map(|&(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct DataTarget {
    pub name: String,
    pub val: f64,
    pub snap_downwards: bool,
    pub snap_upwards: bool,
}

impl FindOrd<Data, Error> for DataTarget {
    fn cmp(&self, t: &Data) -> Result<FindOrdering, Error> {
        match t.get(&self.name) {
            None => Err(anyhow!("missing key '{}'", self.name)),
            Some(&val) => Ok(
                if self.val < val {
                    FindOrdering::ValAboveTarget { is_valid_res: self.snap_upwards }
                } else if self.val > val {
                    FindOrdering::ValBelowTarget { is_valid_res: self.snap_downwards }
                } else {
                    FindOrdering::ValMatchesTarget
                }
            ),
        }
    }
}

impl DataTarget {
    pub fn from_pair(p: Pair, t: Target) -> Result<DataTarget, Error> {
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
        Ok(DataTarget { name, val, snap_downwards, snap_upwards })
    }
}

#[derive(Clone)]
pub enum Target {
    Lower,
    Upper,
}
