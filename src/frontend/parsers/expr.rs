use std::sync::LazyLock;

use enum_map::{Enum, EnumMap, enum_map};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::char;
use nom::character::complete::{alpha1, alphanumeric1, anychar, i32, multispace0};
use nom::combinator::{cut, fail, map, recognize, value};
use nom::multi::{many0, many0_count, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{IResult, Parser};

use crate::frontend::parsers::common::precedence::{
    Assoc, Operation, binary_op, precedence, unary_op,
};
use crate::frontend::parsers::common::string::parse_string;
use crate::frontend::parsers::common::{parse_ident, ws};
use crate::frontend::parsers::stmt::{Stmt, parse_stmt};

#[derive(Clone, Debug, PartialEq, Eq, Enum)]
enum UnaryOpMode
{
    Neg,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Enum)]
enum BinOpMode
{
    Add,
    Sub,
    Mul,
    Div,
}

static UNARY_OP_SYMS: LazyLock<EnumMap<UnaryOpMode, (usize, &'static str)>> = LazyLock::new(|| {
    use UnaryOpMode::*;

    enum_map! {
        Neg => (1, "-")
    }
});

static BIN_OP_SYMS: LazyLock<EnumMap<BinOpMode, (usize, Assoc, &'static str)>> =
    LazyLock::new(|| {
        use BinOpMode::*;

        enum_map! {
            Add => (3, Assoc::Left, "+"),
            Sub => (3, Assoc::Left, "-"),
            Mul => (2, Assoc::Left, "*"),
            Div => (2, Assoc::Left, "/")
        }
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal
{
    Int(i32),
    Char(char),
    Bool(bool),
    String(String),
}

type _Expr<'a> = Box<Expr<'a>>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr<'a>
{
    Literal(Literal),
    Ident(&'a str),
    Call(&'a str, Vec<Expr<'a>>),
    UnaryOp(UnaryOpMode, _Expr<'a>),
    BinaryOp(BinOpMode, _Expr<'a>, _Expr<'a>),
    Stmt(Stmt<'a>),
    Block(Vec<Expr<'a>>),
}

fn parse_literal(input: &str) -> IResult<&str, Expr>
{
    alt((
        map(i32, |x| Literal::Int(x)),
        map(delimited(char('\''), anychar, char('\'')), |c| {
            Literal::Char(c)
        }),
        map(
            alt((value(true, tag("true")), value(false, tag("false")))),
            |b| Literal::Bool(b),
        ),
        map(parse_string, |s| Literal::String(s)),
    ))
    .parse(input)
    .map(|(rem, l)| (rem, Expr::Literal(l)))
}

fn parse_call(input: &str) -> IResult<&str, Expr>
{
    (
        parse_ident,
        ws(delimited(char('('), separated_list0(ws(char(',')), parse_expr), char(')')))
    )
    .map(|(id, params)| Expr::Call(id, params))
    .parse(input)
}

fn parse_sub_expr(input: &str) -> IResult<&str, Expr>
{
    ws(delimited(
        char('('),
        preceded(multispace0, parse_expr),
        cut(preceded(multispace0, char(')'))),
    ))
    .parse(input)
}

pub fn parse_block(input: &str) -> IResult<&str, Expr>
{
    ws(delimited(
        char('{'),
        ws(separated_list0(ws(char(';')), parse_expr)),
        char('}'),
    ))
    .map(|x| Expr::Block(x))
    .parse(input)
}

pub fn parse_expr(input: &str) -> IResult<&str, Expr>
{
    precedence(
        alt(UNARY_OP_SYMS
            .map(|k, (p, v)| unary_op(p, value(k, tag(v))))
            .into_array()),
        fail(),
        alt(BIN_OP_SYMS
            .map(|k, (p, a, v)| binary_op(p, a, value(k, tag(v))))
            .into_array()),
        ws(alt((
            parse_literal,
            parse_ident.map(|id| Expr::Ident(id)),
            parse_sub_expr,
            parse_block,
            parse_stmt.map(|x| Expr::Stmt(x)),
        ))),
        |op: Operation<UnaryOpMode, UnaryOpMode, BinOpMode, Expr>| {
            use Operation::*;

            match op {
                Prefix(mode, e) => Ok(Expr::UnaryOp(mode, Box::new(e))),
                Postfix(e, mode) => Ok(Expr::UnaryOp(mode, Box::new(e))),
                Binary(e1, mode, e2) => Ok(Expr::BinaryOp(mode, Box::new(e1), Box::new(e2))),
                _ => Err("Invalid Combination"),
            }
        },
    )
    .parse(input)
}

#[cfg(test)]
mod expr_test
{
    use super::*;

    #[test]
    fn parse_empty_block()
    {
        assert_eq!(parse_block("{  \n   }").unwrap().1, Expr::Block(vec!()))
    }

    #[test]
    fn literal_basic_test()
    {
        // Integers
        assert_eq!(
            parse_literal("32").unwrap().1,
            Expr::Literal(Literal::Int(32))
        );

        assert_eq!(
            parse_literal("-32").unwrap().1,
            Expr::Literal(Literal::Int(-32))
        );

        // Strings
        assert_eq!(
            parse_literal("\"This is some really interesting text\"")
                .unwrap()
                .1,
            Expr::Literal(Literal::String(String::from(
                "This is some really interesting text"
            )))
        );

        assert!(parse_literal("This is some text").is_err())
    }

    #[test]
    fn complex_expr_test()
    {
        assert_eq!(
            parse_expr("-(5 + 4)").unwrap().1,
            Expr::UnaryOp(
                UnaryOpMode::Neg,
                Box::new(Expr::BinaryOp(
                    BinOpMode::Add,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(4)))
                ))
            )
        )
    }
}
