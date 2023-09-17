mod target;
mod pair;

use b_section::find::{Element, find, FindOrd};
use b_section::find_range::find_range;
use std::collections::HashMap;
use std::io::stdin;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use b_section::combine::{FindOrdCombineLower, FindOrdCombineUpper};
use crate::pair::{Op, Pair};
use crate::target::{Data, DataTarget, Target};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long = "from", help = "Lower constraint.")]
    from: Vec<String>,
    #[clap(long = "to", help = "Upper constraint.")]
    to: Vec<String>,
}

fn parse_stdin_records(s: String) -> Result<HashMap<String, f64>> {
    s.split(" ")
        .map(|p| {
            let Pair { name, op, value } = Pair::parse(p)?;
            if op != Op::Equals {
                return Err(anyhow!("invalid op '{:?}'", op));
            }
            Ok((name, value.parse()?))
        })
        .collect()
}


pub fn new_lookup(datas: &Vec<Data>) -> impl Fn(i64) -> Result<Data> + '_ {
    |idx| {
        if idx < 0 {
            return Err(anyhow!("negative index {}", idx));
        }
        let idx = idx as usize;
        if idx >= datas.len() {
            return Err(anyhow!("index {} out of bounds", idx));
        }
        Ok(datas[idx].clone())
    }
}

fn map_to_pairs(ss: Vec<String>) -> Result<Vec<Pair>> {
    ss.iter().map(|s| s.as_str()).map(Pair::parse).collect()
}

fn map_to_targets<E>(ps: Vec<Pair>, t: Target) -> Result<Vec<Box<DataTarget>>> {
    ps.iter().map(|p| DataTarget::from_pair::<E>(p.clone(), t.clone())).collect()
}

fn resolve_snap(ds: &Vec<Box<DataTarget>>) -> Option<(Vec<Box<dyn FindOrd<Data, anyhow::Error>>>, bool, bool)> {
    let mut snap_downwards = None;
    let mut snap_upwards = None;
    let mut fs: Vec<Box<dyn FindOrd<Data, anyhow::Error>>> = Vec::new(); // converting element type
    for f in ds.iter() {
        fs.push(f.clone());
        let d = f.snap_downwards;
        if let Some(s) = snap_downwards {
            if s != d {
                return None;
            }
        } else {
            snap_downwards = Some(d);
        }
        let u = f.snap_upwards;
        if let Some(s) = snap_upwards {
            if s != u {
                return None;
            }
        } else {
            snap_upwards = Some(u);
        }
    }
    snap_downwards.zip(snap_upwards).map(|(d, u)|(fs, d, u))
}

fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();
    let from = args.from;
    let to = args.to;

    let lower_target_combined = map_to_targets::<anyhow::Error>(map_to_pairs(from)?, Target::Lower)?;
    let upper_target_combined = map_to_targets::<anyhow::Error>(map_to_pairs(to)?, Target::Upper)?;
    let lower_target =
        if lower_target_combined.is_empty() {
            None
        } else {
            // TODO Add context to error.
            let (combined, snap_downwards, snap_upwards) = resolve_snap(&lower_target_combined).ok_or(anyhow!("bla bla 1"))?;
            Some(FindOrdCombineUpper { combined, snap_downwards, snap_upwards })
        };
    let upper_target =
        if upper_target_combined.is_empty() {
            None
        } else {
            // TODO Add context to error.
            let (combined, snap_downwards, snap_upwards) = resolve_snap(&lower_target_combined).ok_or(anyhow!("bla bla 2"))?;
            Some(FindOrdCombineLower { combined, snap_downwards, snap_upwards })
        };

    // Parse records from stdin.
    let datas =
        stdin()
            .lines()
            .map(|l| parse_stdin_records(l?).context("cannot parse records on stdin"))
            .collect::<Result<Vec<Data>>>()?;

    // Run bisection.
    let (lower, upper) = match (lower_target, upper_target) {
        (Some(lt), Some(ut)) =>
            find_range(
                &new_lookup(&datas),
                &lt,
                &ut,
                0,
                datas.len() as i64 - 1,
            )?,
        (Some(t), None) =>
            (
                find(
                    &new_lookup(&datas),
                    &t,
                    0,
                    datas.len() as i64 - 1,
                )?.element,
                None,
            ),
        (None, Some(t)) =>
            (
                None,
                find(
                    &new_lookup(&datas),
                    &t,
                    0,
                    datas.len() as i64 - 1,
                )?.element,
            ),
        (None, None) => (None, None),
    };

    // Print results.
    if let Some(Element { val, idx }) = lower {
        println!("LOWER: index {}: {:?}", idx, val);
    } else {
        println!("LOWER: none!");
    }
    if let Some(Element { val, idx }) = upper {
        println!("UPPER: index {}: {:?}", idx, val)
    } else {
        println!("UPPER: none!");
    }
    Ok(())
}
