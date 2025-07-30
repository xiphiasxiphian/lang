use std::ops::Add;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::opt,
    sequence::{delimited, preceded},
};

use crate::frontend::parsers::{
    common::{Ident, parse_ident, ws},
    expr::{Expr, parse_block, parse_expr},
    types::{Type, parse_type},
};

type _Expr<'a> = Box<Expr<'a>>;
type _Stmt<'a> = Box<Stmt<'a>>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt<'a>
{
    Assign
    {
        id: Ident<'a>,
        ty: Option<Type>,
        rvalue: _Expr<'a>,
    },
    If
    {
        cond: _Expr<'a>,
        tt: _Expr<'a>,
        ff: Option<_Expr<'a>>,
    },
    While
    {
        cond: _Expr<'a>, then: _Expr<'a>
    },
}

fn parse_assign(input: &str) -> IResult<&str, Stmt>
{
    alt((
        (
            preceded(ws(tag("let")), parse_ident),
            opt(preceded(ws(char(':')), parse_type)),
            preceded(ws(char('=')), parse_expr),
        ),
        (parse_ident, preceded(ws(char('=')), parse_expr)).map(|(a, b)| (a, None, b)),
    ))
    .map(|(id, ty, ex)| Stmt::Assign {
        id,
        ty,
        rvalue: Box::new(ex),
    })
    .parse(input)
}

fn parse_if(input: &str) -> IResult<&str, Stmt>
{
    (
        preceded(
            ws(tag("if")),
            delimited(ws(char('(')), parse_expr, ws(char(')'))),
        ),
        parse_block,
        opt(preceded(ws(tag("else")), parse_block)),
    )
        .map(|(cond, tt, ff)| Stmt::If {
            cond: Box::new(cond),
            tt: Box::new(tt),
            ff: ff.map(|x| Box::new(x)),
        })
        .parse(input)
}

fn parse_while(input: &str) -> IResult<&str, Stmt>
{
    (
        preceded(
            ws(tag("while")),
            delimited(ws(char('(')), parse_expr, ws(char(')'))),
        ),
        parse_block,
    )
        .map(|(cond, then)| Stmt::While {
            cond: Box::new(cond),
            then: Box::new(then),
        })
        .parse(input)
}

pub fn parse_stmt(input: &str) -> IResult<&str, Stmt>
{
    alt((parse_assign, parse_if, parse_while)).parse(input)
}

#[cfg(test)]
mod stmt_tests
{
    use super::*;
    use crate::frontend::parsers::{expr::Literal, types::BasicType};

    #[test]
    fn basic_assignments()
    {
        assert_eq!(
            parse_assign("let a: int = 45").unwrap().1,
            Stmt::Assign {
                id: "a",
                ty: Some(Type::BasicType(BasicType::Int)),
                rvalue: Box::new(Expr::Literal(Literal::Int(45)))
            }
        );

        assert_eq!(
            parse_assign("a = 45").unwrap().1,
            Stmt::Assign {
                id: "a",
                ty: None,
                rvalue: Box::new(Expr::Literal(Literal::Int(45)))
            }
        );

        assert!(parse_assign("a: int = 45").is_err())
    }

    #[test]
    fn basic_if()
    {
        assert_eq!(
            parse_if("if (true) { 32 } else { 42 }").unwrap().1,
            Stmt::If {
                cond: Box::new(Expr::Literal(Literal::Bool(true))),
                tt: Box::new(Expr::Block(
                    vec!(),
                    Some(Box::new(Expr::Literal(Literal::Int(32))))
                )),
                ff: Some(Box::new(Expr::Block(
                    vec!(),
                    Some(Box::new(Expr::Literal(Literal::Int(42))))
                ))),
            }
        );

        assert_eq!(
            parse_if("if (false) { 50 }").unwrap().1,
            Stmt::If {
                cond: Box::new(Expr::Literal(Literal::Bool(false))),
                tt: Box::new(Expr::Block(
                    vec!(),
                    Some(Box::new(Expr::Literal(Literal::Int(50))))
                )),
                ff: None
            }
        );

        assert_eq!(
            parse_if("if (false) {  }").unwrap().1,
            Stmt::If {
                cond: Box::new(Expr::Literal(Literal::Bool(false))),
                tt: Box::new(Expr::Block(vec!(), None)),
                ff: None
            }
        )
    }
}
