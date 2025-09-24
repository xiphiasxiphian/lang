use nom::{branch::alt, character::complete::char, multi::{fold_many0, many0, many1}, sequence::delimited, Parser};

use crate::frontend::{parsers::{common::{parse_ident}, expr::{parse_expr, Expr}, ParseResult, Span}, Ident};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LValue
{
    Ident(Ident),
    ArrayElem(Box<LValue>, Box<Expr>)
}


pub fn parse_lvalue(input: Span) -> ParseResult<LValue>
{
    (
        parse_ident,
        many0(
            delimited(
                char('['),
                parse_expr,
                char(']')
            )
        )
    )
    .map(|(id, exs)|
        exs.into_iter().fold(LValue::Ident(id), |x, y| LValue::ArrayElem(Box::new(x), Box::new(y)))
    )
    .parse(input)
}
