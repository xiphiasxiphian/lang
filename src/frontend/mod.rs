use std::{cell::RefCell, rc::Rc};

use crate::frontend::{
    errors::CompileError,
    parsers::parse_prog,
    semantic::{SemProg, semantic_check},
};

pub mod errors;
pub mod parsers;
pub mod semantic;

pub type Ident = String;
pub type Errors = Vec<CompileError>;
pub type ErrorBuffer = Rc<RefCell<Errors>>;

pub fn frontend(input: &str) -> Result<SemProg, Vec<CompileError>>
{
    // Parsing
    let prog = parse_prog(input)?;

    // Semantic Checking
    let sem_prog = semantic_check(&prog)?;

    Ok(sem_prog)
}
