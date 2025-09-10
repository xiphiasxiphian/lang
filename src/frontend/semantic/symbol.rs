use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use crate::frontend::parsers::types::Type;

const SPECIAL_CHAR: char = '$';
pub type UniqueId = String;

pub fn gen_id(id: String, num: usize) -> UniqueId
{
    format!("{id}{SPECIAL_CHAR}{num}")
}


#[derive(Clone)]
pub struct FunctionTypeInfo
{
    pub params: Vec<Type>,
    pub return_type: Type,
}

type FunctionTable = HashMap<UniqueId, FunctionTypeInfo>;
type GlobalTable = HashMap<UniqueId, Type>;

pub type SymbolTableBuffer = Rc<RefCell<SymbolTable>>;

#[derive(Default)]
pub struct SymbolTable
{
    pub funcs: FunctionTable,
    pub globals: GlobalTable,
    pub undefined: HashSet<UniqueId>,
}

impl SymbolTable
{

    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn new_buffer() -> SymbolTableBuffer
    {
        Rc::new(RefCell::new(Self::new()))
    }

    pub fn from_buffer(buffer: SymbolTableBuffer) -> Self
    {
        buffer.take()
    }

    pub fn get_global(&self, id: &UniqueId) -> Option<&Type>
    {
        self.globals.get(id)
    }
}
