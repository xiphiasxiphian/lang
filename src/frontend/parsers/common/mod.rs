pub mod string;

use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::{delimited, pair};
use nom::{AsBytes, AsChar, IResult, Input, Offset, Parser};
use nom_locate::LocatedSpan;
use nom_supreme::tag::complete::tag;

use crate::frontend::Ident;
use crate::frontend::parsers::{ParseResult, Span};

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
    // Currently allows for keywords as identifiers. TODO: Correct this
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .map_res(|x: Span| String::from_str(x.fragment()))
    .parse(input)
}
