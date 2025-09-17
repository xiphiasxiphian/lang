use std::{cell::RefCell, rc::Rc};

use crate::frontend::{
    errors::CompileError, parsers::{func::Func, Prog}, semantic::{scope::Scopes, symbol::SymbolTable, types::TypeChecker}, Errors
};

pub mod scope;
pub mod symbol;
pub mod types;

pub struct SemProg
{
    funcs: Vec<Func>,
    symbols: SymbolTable,
}

pub fn semantic_check(prog: &Prog) -> Result<SemProg, Vec<CompileError>>
{
    // Setup
    let symbols_buffer = SymbolTable::new_buffer();
    let error_buffer = Rc::new(RefCell::new(Errors::new()));

    // Scope Checking
    let new_prog = Scopes::eval_prog(prog, error_buffer.clone(), symbols_buffer.clone());

    let mut symbols = SymbolTable::from_buffer(symbols_buffer)?;

    // Type Checking
    TypeChecker::new(error_buffer.borrow_mut().as_mut(), &mut symbols).check_prog(&new_prog);

    let errors = error_buffer.take();
    if errors.is_empty()
    {
        Ok(SemProg {
            funcs: new_prog.funcs,
            symbols: symbols,
        })
    }
    else
    {
        Err(errors)
    }
}
