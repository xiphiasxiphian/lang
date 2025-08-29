use crate::frontend::frontend;

mod common;
mod frontend;

fn compile(input: &str)
{
    let syn_prog = frontend(input).expect("TODO: Havent fix compile function get");
}

fn main()
{
    let prog = compile("test");
}
