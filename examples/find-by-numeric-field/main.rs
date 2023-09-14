mod target;
mod pair;

use b_section::find::Element;
use b_section::find_range::find_range;
use std::collections::HashMap;
use std::io::stdin;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crate::pair::{Op, Pair};
use crate::target::{Data, DataTarget, Target};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long = "from", help = "Lower constraint.")]
    from: String,
    #[clap(long = "to", help = "Upper constraint.")]
    to: String,
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

fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();
    let from = args.from;
    let to = args.to;

    let from_pair = Pair::parse(from.as_str())?;
    let to_pair = Pair::parse(to.as_str())?;

    // Parse records from stdin.
    let datas =
        stdin()
            .lines()
            .map(|l| parse_stdin_records(l?).context("cannot parse records on stdin"))
            .collect::<Result<Vec<Data>>>()?;

    // Run bisection.
    let (lower, upper) = find_range(
        &new_lookup(&datas),
        DataTarget::from_pair(from_pair, Target::Lower)?.as_ref(),
        DataTarget::from_pair(to_pair, Target::Upper)?.as_ref(),
        0,
        datas.len() as i64 - 1,
    )?;

    // Print results.
    if let Some(Element { val, idx }) = lower {
        println!("LOWER: index {}: {:?}", idx, val)
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
