use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Equals,
    Tilde,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub name: String,
    pub op: Op,
    pub value: String,
}

impl Pair {
    pub fn parse(s: &str) -> Result<Pair> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\w+)([=~])(.+)$").unwrap();
        }
        match RE.captures(s) {
            None => Err(anyhow!("invalid pair '{}'", s)),
            Some(c) => Ok(Pair {
                name: c[1].to_string(),
                op: match c[2].to_string().as_str() {
                    "=" => Op::Equals,
                    "~" => Op::Tilde,
                    x => return Err(anyhow!("invalid op '{}'", x))
                },
                value: c[3].to_string(),
            }),
        }
    }
}

