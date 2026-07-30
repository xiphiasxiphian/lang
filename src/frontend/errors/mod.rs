use std::fmt::Display;

use itertools::Itertools;
use nom::error::ErrorKind;
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation};

use crate::{
    common::ScopeMethods,
    frontend::{
        errors::builder::{ErrorBuilder, LineAttachment, SourceLine},
        parsers::{Span, types::Type},
        semantic::types::TypeContraint,
    },
};

pub mod builder;

type ErrorVariant = BaseErrorKind<&'static str, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

#[derive(Clone, Debug)]
pub struct CompileError
{
    builder: ErrorBuilder,
}

impl CompileError
{
    pub fn format(self) -> String { self.builder.result() }

    // List of Error Type Generators
    pub fn blank_error() -> Self
    {
        Self {
            builder: ErrorBuilder::new(),
        }
    }

    pub fn raw_error(raw: String) -> Self
    {
        Self {
            builder: ErrorBuilder::new().also_mut(|x| {
                x.with_summary(raw);
            }),
        }
    }

    pub fn syntax_error(from: Span, context: Option<(String, String)>) -> Self
    {
        let (summary, reason) = context.unwrap_or_else(|| {
            (
                format!("Error while parsing \"{}\"", from.fragment()),
                "Error while parsing here".into(),
            )
        });

        let raw_line: String = str::from_utf8(from.get_line_beginning())
            .expect("Failed to convert bytes into string")
            .into();
        let col = from.get_utf8_column();
        let range = from.fragment().len().scope(|x| (col - 1)..(col + x - 1));

        let line = SourceLine::new(raw_line, from.location_line() as usize).also_mut(|x| {
            x.add_attachment(LineAttachment::Highlight('^', range, Some(reason)));
        });

        let builder = ErrorBuilder::new().also_mut(|x| {
            x.with_type("Syntax Error".into())
                .with_summary(summary)
                .with_location("".into(), from.location_line() as usize, col)
                .with_source_line(line);
        });

        Self { builder }
    }

    pub fn type_error(from: Span, constraint: TypeContraint, found: Type)
    {
        let expectation = match constraint
        {
            TypeContraint::Is(ty) => format!("which doesn't match expected {ty}"),
            TypeContraint::IsComparibleTo(ty) => format!("which is not comparible to {ty}"),
            _ => unreachable!(),
        };

        let summary = format!("Unexpected type {found}");
        let reason = format!("Found {found} here, {expectation}");

        let raw_line: String = str::from_utf8(from.get_line_beginning())
            .expect("Failed to convert bytes into string")
            .into();
        let col = from.get_utf8_column();
        let range = from.fragment().len().scope(|x| (col - 1)..(col + x - 1));

        let line = SourceLine::new(raw_line, from.location_line() as usize).also_mut(|x| {
            x.add_attachment(LineAttachment::Highlight('^', range, Some(reason)));
        });
    }

    // Helpers for converting ErrorTree into CompileError

    fn expectation_format<T1, T2>(expected: T1, found: T2) -> (String, String)
    where
        T1: Display,
        T2: Display,
    {
        (
            format!("Expected {expected} but found {found} instead"),
            format!("Expected {expected} here"),
        )
    }

    fn translate_error_kind(error: ErrorVariant) -> Option<String>
    {
        match error
        {
            ErrorVariant::Expected(Expectation::Char(c)) => Some(format!("'{c}'")),
            ErrorVariant::Expected(Expectation::Tag(s)) => Some(format!("\"{s}\"")),
            ErrorVariant::Expected(e) => Some(e.to_string()),
            ErrorVariant::Kind(kind) => match kind
            {
                ErrorKind::Alpha => Some(Expectation::<Span>::Alpha.to_string()),
                ErrorKind::Digit => Some(Expectation::<Span>::Digit.to_string()),
                ErrorKind::AlphaNumeric => Some(Expectation::<Span>::AlphaNumeric.to_string()),
                ErrorKind::BinDigit => Some("a binary digit".into()),
                ErrorKind::Float => Some("a float".into()),
                ErrorKind::HexDigit => Some(Expectation::<Span>::HexDigit.to_string()),
                ErrorKind::MultiSpace | ErrorKind::Space => Some("whitespace".into()),
                ErrorKind::Char => Some("a character".into()),
                ErrorKind::OctDigit => Some(Expectation::<Span>::OctDigit.to_string()),
                ErrorKind::Eof => Some("the end of the file".into()),
                _ => None,
            },
            ErrorVariant::External(_) => None,
        }
    }

    fn tree_base(loc: Span, error: ErrorVariant) -> Self
    {
        Self::syntax_error(
            loc,
            Self::translate_error_kind(error).map(|x| Self::expectation_format(x, loc.fragment())),
        )
    }

    fn format_list<T: Display>(items: Vec<T>, ending: &str) -> String
    {
        const MAX: usize = 4;
        let length = items.len();

        match length
        {
            0 => "".into(),
            1 => items.first().unwrap().to_string(),
            l if l <= MAX => format!(
                "{} {} {}",
                items.iter().take(l - 1).join(","),
                ending,
                items.last().unwrap()
            ),
            _ => format!("{} ...", items.iter().take(MAX).join(", ")),
        }
    }

    fn tree_alt(alts: Vec<ErrorTree<Span>>) -> Self
    {
        let results: Vec<(Span, String)> = alts
            .into_iter()
            .map(|x| match x
            {
                ErrorTree::Base { location, kind } => (location, Self::translate_error_kind(kind).unwrap()),
                _ => todo!(),
            })
            .collect();

        Self::syntax_error(
            results[0].0,
            Some(Self::expectation_format(
                Self::format_list(results.iter().map(|x| x.1.clone()).collect(), "or"),
                results[0].0.fragment(),
            )),
        )
    }
}

impl<'a> From<ErrorTree<Span<'a>>> for CompileError
{
    fn from(value: ErrorTree<Span<'a>>) -> Self
    {
        match value
        {
            ErrorTree::Base { location, kind } => Self::tree_base(location, kind),
            ErrorTree::Stack { base, contexts } => Self::from(*base),
            ErrorTree::Alt(bs) => Self::blank_error().also(|_| println!("Alt Error {:?}", bs)),
        }
    }
}
