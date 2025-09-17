pub mod string;

use std::fmt::Display;
use std::str::FromStr;

use enum_map::Enum;
use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{recognize, not};
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::{delimited, pair};
use nom::{AsChar, Input, Parser};
use nom_supreme::tag::complete::tag;

use crate::frontend::parsers::types::BasicType;
use crate::frontend::Ident;
use crate::frontend::parsers::{ParseResult, Span};

#[derive(Enum, Clone, Debug)]
pub enum Keyword
{
    Cond,
    CondElse,
    Loop,
    Decl,
    Type(BasicType),
    Func,
}

impl From<Keyword> for &str
{
    fn from(value: Keyword) -> Self {
        use Keyword::*;
        match value
        {
            Cond => "if",
            CondElse => "else",
            Loop => "while",
            Decl => "let",
            Type(t) => t.as_str(),
            Func => "fun"
        }
    }
}

pub fn ws<I, O, E: ParseError<I>, G>(parser: G) -> impl Parser<I, Output = O, Error = E>
where
    G: Parser<I, Output = O, Error = E>,
    I: Input,
    <I as Input>::Item: AsChar,
{
    delimited(multispace0, parser, multispace0)
}

pub fn parse_ident(input: Span) -> ParseResult<Ident>
{
    // TODO: filter out keywords
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .map_res(|x: Span| String::from_str(x.fragment()))
    .parse(input)
}
