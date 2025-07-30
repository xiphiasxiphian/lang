use nom::{
    IResult, Parser,
    bytes::complete::tag,
    multi::separated_list0,
    sequence::{delimited, preceded},
};

use crate::frontend::parsers::{
    common::{parse_ident, ws},
    expr::{Expr, parse_block},
    types::{Type, parse_type},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Func<'a>
{
    name: &'a str,
    parameters: Vec<(&'a str, Type)>,
    return_type: Type,
    block: Expr<'a>,
}

fn parse_parameter(input: &str) -> IResult<&str, (&str, Type)>
{
    (parse_ident, ws(tag(":")), parse_type)
        .parse(input)
        .map(|(rem, (id, _, ty))| (rem, (id, ty)))
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
        .parse(input)
        .map(|(rem, (name, params, ty, block))| {
            (
                rem,
                Func {
                    name,
                    parameters: params,
                    return_type: ty,
                    block,
                },
            )
        })
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
                name: "foo",
                parameters: vec![
                    ("a", Type::BasicType(BasicType::Int)),
                    ("b", Type::BasicType(BasicType::Int))
                ],
                return_type: Type::BasicType(BasicType::Int),
                block: Expr::Block(vec!(), None)
            }
        )
    }
}
