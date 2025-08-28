use nom::error::{ErrorKind, ParseError};

use crate::{
    common::ScopeMethods,
    frontend::{
        ErrorBuffer, Errors,
        errors::builder::{ErrorBuilder, LineAttachment, SourceLine},
        parsers::Span,
    },
};

pub mod builder;

#[derive(Clone)]
pub struct CompileError
{
    builder: ErrorBuilder,
    others: Vec<Self>,
}

impl CompileError
{
    // List of Error Type Generators
    pub fn blank_error() -> Self
    {
        Self {
            builder: ErrorBuilder::new(),
            others: vec![],
        }
    }

    pub fn syntax_error() -> Self
    {
        let builder = ErrorBuilder::new().also_mut(|x| {
            x.with_type("Syntax Error".into()).with_summary("".into());
        });

        Self {
            builder,
            others: vec![],
        }
    }

    pub fn format(self) -> String
    {
        self.builder.result()
    }
}
