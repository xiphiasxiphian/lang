use nom::{
    IResult, Parser,
    bytes::complete::tag,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
};

use crate::frontend::{
    Ident,
    parsers::{
        common::{parse_ident, ws},
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

fn parse_parameter(input: &str) -> IResult<&str, (String, Type)>
{
    separated_pair(parse_ident, ws(tag(":")), parse_type).parse(input)
}

pub fn parse_func(input: &str) -> IResult<&str, Func>
{
    (
        preceded(ws(tag("fun")), parse_ident),
        delimited(
            tag("("),
            separated_list0(ws(tag(",")), parse_parameter),
            tag(")"),
        ),
        preceded(ws(tag("->")), parse_type),
        parse_block,
    )
        .map(|(name, params, ty, block)| Func {
            name: name.into(),
            parameters: params,
            return_type: ty,
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
            parse_func("fun foo(a: int, b: int) -> int {}").unwrap().1,
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
