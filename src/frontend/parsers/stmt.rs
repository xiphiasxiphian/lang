use nom::{
    Parser,
    branch::alt,
    character::complete::char,
    combinator::opt,
    sequence::{delimited, preceded, separated_pair},
};

use nom_supreme::tag::complete::tag;

use crate::frontend::{
    Ident,
    parsers::{
        ParseResult, Span,
        common::{parse_ident, ws},
        expr::{Expr, parse_block, parse_expr},
        types::{Type, parse_type},
    },
};

type _Expr = Box<Expr>;
type _Stmt = Box<Stmt>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt
{
    Declare
    {
        id: Ident,
        ty: Option<Type>,
        rvalue: _Expr,
    },
    Assign
    {
        id: Ident, rvalue: _Expr
    },
    If
    {
        cond: _Expr,
        tt: _Expr,
        ff: Option<_Expr>,
    },
    While
    {
        cond: _Expr, then: _Expr
    },
}

fn parse_declare(input: Span) -> ParseResult<Stmt>
{
    (
        preceded(ws(tag("let")), parse_ident),
        opt(preceded(ws(char(':')), parse_type)),
        preceded(ws(char('=')), parse_expr),
    )
        .map(|(id, ty, ex)| Stmt::Declare {
            id: id.into(),
            ty,
            rvalue: Box::new(ex),
        })
        .parse(input)
}

fn parse_assign(input: Span) -> ParseResult<Stmt>
{
    separated_pair(parse_ident, ws(char('=')), parse_expr)
        .map(|(id, ex)| Stmt::Assign {
            id: id,
            rvalue: Box::new(ex),
        })
        .parse(input)
}

fn parse_if(input: Span) -> ParseResult<Stmt>
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

fn parse_while(input: Span) -> ParseResult<Stmt>
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

pub fn parse_stmt(input: Span) -> ParseResult<Stmt>
{
    alt((parse_declare, parse_assign, parse_if, parse_while)).parse(input)
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
            parse_stmt(Span::new("let a: int = 45")).unwrap().1,
            Stmt::Declare {
                id: "a".into(),
                ty: Some(Type::BasicType(BasicType::Int)),
                rvalue: Box::new(Expr::Literal(Literal::Int(45)))
            }
        );

        assert_eq!(
            parse_stmt(Span::new("a = 45")).unwrap().1,
            Stmt::Assign {
                id: "a".into(),
                rvalue: Box::new(Expr::Literal(Literal::Int(45)))
            }
        );

        assert!(parse_stmt(Span::new("a: int = 45")).is_err())
    }

    #[test]
    fn basic_if()
    {
        assert_eq!(
            parse_if(Span::new("if (true) { 32 } else { 42 }"))
                .unwrap()
                .1,
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
            parse_if(Span::new("if (false) { 50 }")).unwrap().1,
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
            parse_if(Span::new("if (false) {  }")).unwrap().1,
            Stmt::If {
                cond: Box::new(Expr::Literal(Literal::Bool(false))),
                tt: Box::new(Expr::Block(vec!(), None)),
                ff: None
            }
        )
    }
}
