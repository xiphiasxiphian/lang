use std::collections::HashMap;

use crate::frontend::parsers::types::Type;

const SPECIAL_CHAR: char = '$';
pub type UniqueId = String;

pub fn gen_id(id: &str, num: usize) -> UniqueId
{
    format!("{id}{SPECIAL_CHAR}{num}")
}

pub struct SymbolTable
{
    pub table: HashMap<UniqueId, Type>
}

impl SymbolTable
{
    pub fn new() -> Self
    {
        Self { table: HashMap::new() }
    }
}
