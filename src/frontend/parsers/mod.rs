use nom::{Finish, IResult, Parser, multi::many1};
use nom_locate::LocatedSpan;
use nom_supreme::{error::ErrorTree, ParserExt};

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
    many1(parse_func).complete()
        .parse_complete(span)
        .finish()
        .map(|(_, x)| Prog { funcs: x })
        .map_err(|e| vec![e.into()])
}
