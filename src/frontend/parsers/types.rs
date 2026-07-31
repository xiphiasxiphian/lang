use std::{
    array, fmt::{Debug, Display}, sync::LazyLock,
};

use strum::{EnumCount, VariantArray, VariantNames};
use nom::{Parser, branch::alt, combinator::value, sequence::delimited};
use nom_supreme::{ParserExt, tag::complete::tag};

use crate::frontend::parsers::{ParseResult, Span};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type
{
    Void,
    BasicType(BasicType),
    Array(Box<Type>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumCount, VariantArray, VariantNames)]
#[strum(serialize_all = "camelCase")]
pub enum BasicType
{
    Int,
    Bool,
    Char,
    String,
}

impl Display for BasicType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", Self::NAMES[*self as usize]) }
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
    pub const NAMES: &'static [&'static str] = <Self as VariantNames>::VARIANTS;
    pub const VALUES: &'static [Self] = <Self as VariantArray>::VARIANTS;

    pub fn as_str(self) -> &'static str { Self::NAMES[self as usize] }
}

fn parse_basic_type(input: Span) -> ParseResult<Type>
{
    const COUNT: usize = BasicType::COUNT;
    alt(array::from_fn::<_, COUNT, _>(|i| value(BasicType::VALUES[i], tag(BasicType::NAMES[i]))))
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
