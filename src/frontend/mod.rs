use crate::frontend::parsers::Prog;
use crate::frontend::parsers::parse_prog;

pub mod parsers;
pub mod semantic;

pub fn frontend(input: &str) -> Result<Prog, Box<dyn std::error::Error + '_>>
{
    parse_prog(input)
}
