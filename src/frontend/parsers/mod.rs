use nom::{Finish, IResult, Parser, multi::many1};
use nom_locate::LocatedSpan;
use nom_supreme::error::ErrorTree;

use crate::frontend::{
    errors::CompileError,
    parsers::func::{Func, parse_func},
};

pub mod expr;
pub mod func;
pub mod stmt;
pub mod types;

mod common;

pub type Span<'a> = LocatedSpan<&'a str, ()>;
pub type ParseResult<'a, T> = IResult<Span<'a>, T, ErrorTree<Span<'a>>>;

pub struct Prog
{
    pub funcs: Vec<Func>,
}

pub fn parse_prog(input: &str) -> Result<Prog, Vec<CompileError>>
{
    let span = Span::new(input);
    match many1(parse_func).parse_complete(span).finish()
    {
        Ok((_, funcs)) => Ok(Prog { funcs }),
        Err(_) => todo!(),
    }
}
