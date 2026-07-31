use std::array;

use nom::{
    Parser,
    branch::alt,
    character::{
        char,
        complete::{anychar, i32, multispace0},
    },
    combinator::{cut, fail, opt, value},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, terminated},
};
use nom_language::precedence::{Assoc, Operation, binary_op, precedence, unary_op};
use nom_supreme::{parser_ext::ParserExt, tag::complete::tag};
use strum::{EnumCount, VariantArray};

use crate::frontend::parsers::{
    ParseResult, Span, common::{parse_ident, string::span_parse_string, ws}, lvalue::{LValue, parse_lvalue}, stmt::{Stmt, parse_stmt}, types::BasicType,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumCount, VariantArray)]
pub enum UnaryOpMode
{
    Neg,
    Not,
}

impl UnaryOpMode
{
    pub const SYMS: [(usize, &'static str); Self::COUNT] = [
        (1, "-"), // Neg
        (1, "!"), // Not
    ];
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumCount, VariantArray)]
pub enum BinOpMode
{
    Add,
    Sub,
    Mul,
    Div,
}

impl BinOpMode
{
    pub const SYMS: [(usize, Assoc, &'static str); Self::COUNT] = [
        (3, Assoc::Left, "+"), // add
        (3, Assoc::Left, "-"), // sub
        (2, Assoc::Left, "*"), // mul
        (2, Assoc::Left, "/"), // div
    ];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal
{
    Int(i32),
    Char(char),
    Bool(bool),
    String(String),
}

type _Expr = Box<Expr>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr
{
    Literal(Literal),
    LValue(LValue),
    Array(Vec<Expr>),
    Call(String, Vec<Expr>),
    UnaryOp(UnaryOpMode, _Expr),
    BinaryOp(BinOpMode, _Expr, _Expr),
    Stmt(Stmt),
    Block(Vec<Expr>, Option<_Expr>),
}

fn parse_literal(input: Span) -> ParseResult<Expr>
{
    alt((
        i32.map(|x| Literal::Int(x)),
        alt((tag("true").value(true), tag("false").value(false))).map(|b| Literal::Bool(b)),
        delimited(char('\''), anychar, char('\'')).map(|c| Literal::Char(c)),
        span_parse_string.map(|s| Literal::String(s)),
    ))
    .map(|l| Expr::Literal(l))
    .parse(input)
}

fn parse_array(input: Span) -> ParseResult<Expr>
{
    ws(delimited(
        char('['),
        ws(separated_list0(ws(char(',')), parse_expr)),
        char(']'),
    ))
    .map(|xs| Expr::Array(xs))
    .parse(input)
}

fn parse_call(input: Span) -> ParseResult<Expr>
{
    (
        parse_ident,
        ws(delimited(
            char('('),
            separated_list0(ws(char(',')), parse_expr),
            char(')'),
        )),
    )
        .map(|(id, params)| Expr::Call(id.into(), params))
        .parse(input)
}

fn parse_sub_expr(input: Span) -> ParseResult<Expr>
{
    ws(delimited(
        char('('),
        preceded(multispace0, parse_expr),
        cut(preceded(multispace0, char(')'))),
    ))
    .parse(input)
}

pub fn parse_block(input: Span) -> ParseResult<Expr>
{
    ws(delimited(
        char('{'),
        ws((ws(many0(terminated(parse_expr, ws(char(';'))))), ws(opt(parse_expr)))),
        char('}'),
    ))
    .map(|(exs, ret)| Expr::Block(exs, ret.map(|x| Box::new(x))))
    .parse(input)
}

pub fn parse_expr(input: Span) -> ParseResult<Expr>
{
    const UNARY_COUNT: usize = UnaryOpMode::COUNT;
    const BINARY_COUNT: usize = BinOpMode::COUNT;

    precedence(
        alt(array::from_fn::<_, UNARY_COUNT, _>(|i| {
            let (k, (p, v)) = (UnaryOpMode::VARIANTS[i], UnaryOpMode::SYMS[i]);
            unary_op(p, value(k, tag(v)))
        })),
        fail(),
        alt(array::from_fn::<_, BINARY_COUNT, _>(|i| {
            let (k, (p, a, v)) = (BinOpMode::VARIANTS[i], BinOpMode::SYMS[i]);
            binary_op(p, a, value(k, tag(v)))
        })),
        ws(alt((
            parse_literal,
            parse_call,
            parse_sub_expr,
            parse_block,
            parse_array,
            parse_stmt.map(|x| Expr::Stmt(x)),
            parse_lvalue.map(|x| Expr::LValue(x)),
        ))),
        |op: Operation<UnaryOpMode, UnaryOpMode, BinOpMode, Expr>| {
            use Operation::*;

            match op
            {
                Prefix(mode, e) => Ok::<Expr, &str>(Expr::UnaryOp(mode, Box::new(e))),
                Postfix(e, mode) => Ok(Expr::UnaryOp(mode, Box::new(e))),
                Binary(e1, mode, e2) => Ok(Expr::BinaryOp(mode, Box::new(e1), Box::new(e2))),
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
    fn parse_empty_block() { assert_eq!(parse_block("{  \n   }".into()).unwrap().1, Expr::Block(vec!(), None)) }

    #[test]
    fn literal_basic_test()
    {
        // Integers
        assert_eq!(parse_literal("32".into()).unwrap().1, Expr::Literal(Literal::Int(32)));

        assert_eq!(parse_literal("-32".into()).unwrap().1, Expr::Literal(Literal::Int(-32)));

        // Strings
        assert_eq!(
            parse_literal(Span::new("\"This is some really interesting text\""))
                .unwrap()
                .1,
            Expr::Literal(Literal::String(String::from("This is some really interesting text")))
        );

        assert!(parse_literal(Span::new("This is some text")).is_err())
    }

    #[test]
    fn complex_expr_test()
    {
        assert_eq!(
            parse_expr(Span::new("-(5 + 4)")).unwrap().1,
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
