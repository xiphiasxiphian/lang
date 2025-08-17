use crate::frontend::{frontend, parsers::Prog};

mod common;
mod frontend;

fn compile(input: &str) -> Result<Prog, Box<dyn std::error::Error + '_>>
{
    let syn_prog = frontend(input)?;

    Ok(syn_prog)
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let prog = compile("test");
    Ok(())
}
