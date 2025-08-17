use std::collections::HashMap;

use crate::frontend::parsers::types::Type;

const SPECIAL_CHAR: char = '$';
pub type UniqueId = String;

pub fn gen_id(id: String, num: usize) -> UniqueId
{
    format!("{id}{SPECIAL_CHAR}{num}")
}

pub struct SymbolTable
{
    table: HashMap<UniqueId, Type>,
}

impl SymbolTable
{
    pub fn new() -> Self
    {
        Self {
            table: HashMap::new(),
        }
    }

    // To be used when names can be expected to be valid after scope checking
    pub fn get(&self, id: &String) -> Type
    {
        self.table
            .get(id)
            .cloned()
            .expect(format!("Id {id} not found. Did Scope Checkingn Succeed properly?").as_str())
    }
}
