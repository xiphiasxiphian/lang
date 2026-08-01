use std::{fmt::Display, ops::Range, str};

use ariadne::{Color, Label, Report, ReportKind, Source};
use itertools::Itertools;
use nom::error::ErrorKind;
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation};

use crate::{
    common::ScopeMethods,
    frontend::{
        parsers::{Span, types::Type},
        semantic::types::TypeContraint,
    },
};

type ErrorVariant = BaseErrorKind<&'static str, Box<dyn std::error::Error + Send + Sync + 'static >>;

#[derive(Clone, Debug)]
pub struct CompileError
{
    kind: ReportKind<'static>,
    summary: String,
    location: usize,
    labels: Vec<(Range<usize>, String, Option<Color>)>,
    context: Vec<(usize, String)>,
}

impl Default for CompileError
{
    fn default() -> Self
    {
        Self {
            kind: ReportKind::Error,
            summary: "[Error] Compile Error".into(),
            location: 0,
            labels: vec![],
            context: vec![],
        }
    }
}

impl CompileError
{
    pub fn format(&self, filename: &str, source: &str) -> String
    {
        let primary_range = self.location..self.location;

        let mut buf = vec![];
        Report::build(self.kind, (filename, primary_range))
            .with_message(&self.summary)
            .with_labels(
                self.labels.iter().map(|(range, msg, color)| {
                    Label::new((filename, range.clone()))
                        .with_message(msg)
                        .with_color(color.unwrap_or_default())
                })
            )
            .with_labels(
                self.context.iter().map(|(offset, ctx)| {
                    Label::new((filename, *offset..*offset))
                        .with_message(format!("while parsing {ctx}"))
                })
            )
            .finish()
            .write((filename, Source::from(source)), &mut buf)
            .unwrap();

        String::from_utf8(buf).unwrap()
    }

    // List of Error Type Generators
    // pub fn syntax_error(from: Span, context: Option<(String, String)>) -> Self
    // {
    //     let (summary, reason) = context.unwrap_or_else(|| {
    //         (
    //             format!("Error while parsing \"{}\"", from.fragment()),
    //             "Error while parsing here".into(),
    //         )
    //     });

    //     let raw_line: String = str::from_utf8(from.get_line_beginning())
    //         .expect("Failed to convert bytes into string")
    //         .into();
    //     let col = from.get_utf8_column();
    //     let range = from.fragment().len().scope(|x| (col - 1)..(col + x - 1));

    //     let line = SourceLine::new(raw_line, from.location_line() as usize).also_mut(|x| {
    //         x.add_attachment(LineAttachment::Highlight('^', range, Some(reason)));
    //     });

    //     let builder = ErrorBuilder::new()
    //         .with_type("Syntax Error".into())
    //         .with_summary(summary)
    //         .with_location("".into(), from.location_line() as usize, col)
    //         .with_source_line(line);

    //     Self { builder }
    // }

    // pub fn type_error(from: Span, constraint: TypeContraint, found: Type) -> Self
    // {
    //     let expectation = match constraint
    //     {
    //         TypeContraint::Is(ty) => format!("which doesn't match expected {ty}"),
    //         TypeContraint::IsComparibleTo(ty) => format!("which is not comparible to {ty}"),
    //         _ => unreachable!(),
    //     };

    //     let summary = format!("Unexpected type {found}");
    //     let reason = format!("Found {found} here, {expectation}");

    //     let raw_line: String = str::from_utf8(from.get_line_beginning())
    //         .expect("Failed to convert bytes into string")
    //         .into();
    //     let col = from.get_utf8_column();
    //     let range = from.fragment().len().scope(|x| (col - 1)..(col + x - 1));

    //     let line = SourceLine::new(raw_line, from.location_line() as usize).also_mut(|x| {
    //         x.add_attachment(LineAttachment::Highlight('^', range, Some(reason)));
    //     });

    //     Self {
    //         builder: ErrorBuilder::new()
    //             .with_type("Type Error".into())
    //             .with_summary(summary)
    //             .with_location("".into(), from.location_line() as usize, col)
    //             .with_source_line(line)
    //     }
    // }

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

    fn translate_error_kind(error: &ErrorVariant) -> Option<String>
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
        let (summary, label_msg) = Self::translate_error_kind(&error)
            .map(|x| Self::expectation_format(x, loc.fragment()))
            .unwrap_or_else(|| ("Syntax error".into(), "error occurred here".into()));

        CompileError {
            kind: ReportKind::Error,
            summary,
            location: loc.location_offset(),
            labels: vec![(loc.location_offset()..loc.location_offset() + loc.fragment().len(), label_msg, Some(Color::Red))],
            context: vec![],
        }
    }

    fn format_list<T: Display>(items: &[T], ending: &str) -> String
    {
        const MAX: usize = 4;
        match items
        {
            [] => "".into(),
            [item] => item.to_string(),
            [is @ .., end] if is.len() <= MAX => format!(
                "{} {} {}",
                is.iter().join(","),
                ending,
                end
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
                ErrorTree::Base { location, kind } => (location, Self::translate_error_kind(&kind).unwrap()),
                _ => todo!(),
            })
            .collect();

        // Self::syntax_error(
        //     results[0].0,
        //     Some(Self::expectation_format(
        //         Self::format_list(results.iter().map(|x| x.1.clone()).collect(), "or"),
        //         results[0].0.fragment(),
        //     )),
        // )

        todo!()
    }
}

impl<'a> From<ErrorTree<Span<'a>>> for CompileError
{
    fn from(value: ErrorTree<Span<'a>>) -> Self {
        match value {
            ErrorTree::Base { location, kind } => Self::tree_base(location, kind),
            ErrorTree::Stack { base, contexts } => {
            }
            ErrorTree::Alt(alts) => Self::tree_alt(alts),
        }
    }
}
