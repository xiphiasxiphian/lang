use std::cell::RefCell;
use std::rc::Rc;

use crate::frontend::errors::CompileError;
use crate::frontend::parsers::parse_prog;
use crate::frontend::semantic::SemProg;
use crate::frontend::semantic::semantic_check;

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
