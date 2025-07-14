pub mod precedence;
pub mod string;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::{delimited, pair};
use nom::{IResult, Parser};

pub type Ident<'a> = &'a str;

pub fn ws<'a, O, E: ParseError<&'a str>, G>(
    parser: G,
) -> impl Parser<&'a str, Output = O, Error = E>
where
    G: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, parser, multispace0)
}

pub fn parse_ident(input: &str) -> IResult<&str, &str>
{
    // Currently allows for keywords as identifiers. TODO: Correct this
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)
}
