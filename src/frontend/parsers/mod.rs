use nom::{Parser, combinator::complete, multi::many1};

use crate::frontend::parsers::func::{Func, parse_func};

pub mod expr;
pub mod func;
pub mod stmt;
pub mod types;

mod common;

pub struct Prog<'a>
{
    funcs: Vec<Func<'a>>,
}

pub fn parse_prog(input: &str) -> Result<Prog, Box<dyn std::error::Error + '_>>
{
    let (_, funcs) = complete(many1(parse_func)).parse(input)?;

    Ok(Prog { funcs })
}
