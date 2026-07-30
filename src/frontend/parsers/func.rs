use nom::{
    Parser,
    combinator::opt,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
};
use nom_supreme::{ParserExt, tag::complete::tag};

use crate::frontend::{
    Ident,
    parsers::{
        ParseResult, Span,
        common::{Keyword, parse_ident, ws},
        expr::{Expr, parse_block},
        types::{Type, parse_type},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct Func
{
    pub name: Ident,
    pub parameters: Vec<(Ident, Type)>,
    pub return_type: Type,
    pub block: Expr,
}

fn parse_parameter(input: Span) -> ParseResult<(String, Type)>
{
    separated_pair(
        parse_ident,
        ws(tag(":")).context("Expected function parameter to be given a type"),
        parse_type,
    )
    .parse(input)
}

pub fn parse_func(input: Span) -> ParseResult<Func>
{
    (
        preceded(ws(tag(Keyword::Func.into())), parse_ident),
        delimited(tag("("), separated_list0(ws(tag(",")), parse_parameter), tag(")")),
        opt(preceded(ws(tag("->")), parse_type)),
        parse_block,
    )
        .map(|(name, params, ty, block)| Func {
            name: name.into(),
            parameters: params,
            return_type: ty.unwrap_or(Type::Void),
            block,
        })
        .parse(input)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::frontend::parsers::types::BasicType;

    #[test]
    fn basic_fun_header_test()
    {
        assert_eq!(
            parse_func(Span::new("fun foo(a: int, b: int) -> int {}")).unwrap().1,
            Func {
                name: "foo".into(),
                parameters: vec![
                    ("a".into(), Type::BasicType(BasicType::Int)),
                    ("b".into(), Type::BasicType(BasicType::Int))
                ],
                return_type: Type::BasicType(BasicType::Int),
                block: Expr::Block(vec!(), None)
            }
        )
    }
}
