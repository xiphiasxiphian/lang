use std::cell::RefCell;
use std::rc::Rc;

use crate::frontend::errors::CompileError;
use crate::frontend::parsers::Prog;
use crate::frontend::parsers::parse_prog;

pub mod errors;
pub mod parsers;
pub mod semantic;

pub type Ident = String;
pub type Errors = Vec<CompileError>;
pub type ErrorBuffer = Rc<RefCell<Errors>>;

pub fn frontend(input: &str) -> Result<Prog, Box<dyn std::error::Error + '_>>
{
    let prog = parse_prog(input)?;

    // Semantic Checking

    Ok(prog)
}
