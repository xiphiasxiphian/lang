use nom::{combinator::complete, multi::many1, Parser};

use crate::frontend::parsers::func::{parse_func, Func};

pub mod expr;
pub mod func;
pub mod stmt;
pub mod types;

mod common;

pub struct Prog<'a>
{
    funcs: Vec<Func<'a>>
}

pub fn parse_prog(input: &str) -> Result<Prog, Box<dyn std::error::Error + '_>>
{
    let (_, funcs) = complete(many1(parse_func)).parse(input)?;

    Ok(Prog {
        funcs
    })
}
