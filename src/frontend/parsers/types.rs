use std::fmt::Debug;

use std::sync::LazyLock;

use enum_map::{Enum, EnumMap, enum_map};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;
use nom::sequence::delimited;
use nom::{IResult, Parser};

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

static TYPES: LazyLock<EnumMap<BasicType, &'static str>> = LazyLock::new(|| {
    use BasicType::*;

    enum_map! {
        Int => "int",
        Bool => "bool",
        Char => "char",
        String => "string"
    }
});

fn parse_basic_type(input: &str) -> IResult<&str, Type>
{
    alt(TYPES.map(|ty, label| value(ty, tag(label))).into_array())
        .map(|ty| Type::BasicType(ty))
        .parse(input)
}

fn parse_array(input: &str) -> IResult<&str, Type>
{
    delimited(tag("["), parse_type, tag("]"))
        .map(|ty| Type::Array(Box::new(ty)))
        .parse(input)
}

pub fn parse_type(input: &str) -> IResult<&str, Type>
{
    alt((
        value(Type::Void, tag("void")),
        parse_basic_type,
        parse_array,
    ))
    .parse(input)
}
