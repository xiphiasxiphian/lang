use nom::{
    Parser,
    character::complete::char,
    multi::many0,
    sequence::delimited,
};

use crate::frontend::{
    Ident,
    parsers::{
        ParseResult, Span,
        common::parse_ident,
        expr::{Expr, parse_expr},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LValue
{
    Ident(Ident),
    ArrayElem(Box<LValue>, Box<Expr>),
}

pub fn parse_lvalue(input: Span) -> ParseResult<LValue>
{
    (parse_ident, many0(delimited(char('['), parse_expr, char(']'))))
        .map(|(id, exs)| {
            exs.into_iter()
                .fold(LValue::Ident(id), |x, y| LValue::ArrayElem(Box::new(x), Box::new(y)))
        })
        .parse(input)
}
