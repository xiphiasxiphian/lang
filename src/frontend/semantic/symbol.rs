use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::frontend::{Ident, errors::CompileError, parsers::types::Type};

const SPECIAL_CHAR: char = '$';
pub type UniqueId = String;

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
    funcs: FunctionTable,
    globals: GlobalTable,
    undefined: HashSet<UniqueId>,
    untyped: HashSet<UniqueId>,
}

impl SymbolTable
{
    pub fn new() -> Self { Self::default() }

    pub fn new_buffer() -> SymbolTableBuffer { Rc::new(RefCell::new(Self::new())) }

    pub fn from_buffer(buffer: SymbolTableBuffer) -> Result<Self, Vec<CompileError>>
    {
        let result = buffer.take();
        if result.undefined.is_empty()
        {
            Ok(result)
        }
        else
        {
            Err(result.undefined.iter().map(|x| todo!()).collect())
        }
    }

    pub fn insert_func(&mut self, id: UniqueId, info: Option<FunctionTypeInfo>) -> Option<UniqueId>
    {
        match info
        {
            Some(t) =>
            {
                self.undefined.remove(&id);
                self.funcs.insert(id.clone(), t).map(|_| id)
            }
            None =>
            {
                if !self.funcs.contains_key(&id)
                {
                    self.undefined.insert(id);
                }
                None
            }
        }
    }

    pub fn new_global_id(&self, id: Ident) -> UniqueId { format!("{id}{SPECIAL_CHAR}{}", self.globals.len()) }

    pub fn insert_global(&mut self, id: UniqueId, ty: Type) -> Option<UniqueId>
    {
        self.globals.insert(id.clone(), ty).map(|_| id)
    }

    pub fn insert_untyped(&mut self, id: UniqueId) -> Option<UniqueId>
    {
        Some(id.clone()).filter(|_| self.untyped.insert(id.clone()))
    }

    pub fn get_global(&mut self, id: &UniqueId) -> Option<&Type> { self.globals.get(id) }

    pub fn set_untyped(&mut self, id: UniqueId, ty: Type)
    {
        if self.untyped.remove(&id)
        {
            self.globals.insert(id, ty);
        }
    }

    pub fn get_func_info(&self, id: &UniqueId) -> Option<&FunctionTypeInfo> { self.funcs.get(id) }
}
