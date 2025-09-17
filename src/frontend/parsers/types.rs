use std::fmt::{Debug, Display};

use std::sync::LazyLock;

use enum_map::{Enum, EnumMap, enum_map};
use nom::Parser;
use nom::branch::alt;
use nom::combinator::value;
use nom::sequence::delimited;
use nom_supreme::{ParserExt, tag::complete::tag};

use crate::frontend::parsers::{ParseResult, Span};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type
{
    Void,
    BasicType(BasicType),
    Array(Box<Type>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Enum)]
pub enum BasicType
{
    Int,
    Bool,
    Char,
    String,
}

impl Display for BasicType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", TYPES[self.clone()])
    }
}

impl Display for Type
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Type::Void => write!(f, "void"),
            Type::BasicType(bt) => write!(f, "{}", bt),
            Type::Array(inner) => write!(f, "[{}]", inner.as_ref()),
        }
    }
}

impl BasicType
{
    pub fn as_str(self) -> &'static str
    {
        TYPES[self]
    }
}

static TYPES: LazyLock<EnumMap<BasicType, &'static str>> = LazyLock::new(|| {
    use BasicType as B;

    enum_map! {
        B::Int => "int",
        B::Bool => "bool",
        B::Char => "char",
        B::String => "string"
    }
});

fn parse_basic_type(input: Span) -> ParseResult<Type>
{
    alt(TYPES.map(|ty, label| value(ty, tag(label))).into_array())
        .context("Expected a basic type (int, bool, char or string)")
        .map(|ty| Type::BasicType(ty))
        .parse(input)
}

fn parse_array(input: Span) -> ParseResult<Type>
{
    delimited(tag("["), parse_type, tag("]"))
        .map(|ty| Type::Array(Box::new(ty)))
        .parse(input)
}

pub fn parse_type(input: Span) -> ParseResult<Type>
{
    alt((tag("void").value(Type::Void), parse_basic_type, parse_array))
        .context("Expected a type")
        .parse(input)
}
