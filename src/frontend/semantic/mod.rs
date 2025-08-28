use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::frontend::{
    Errors,
    errors::CompileError,
    parsers::{Prog, func::Func, types::Type},
    semantic::{scope::Scopes, symbol::UniqueId, types::TypeChecker},
};

pub mod scope;
pub mod symbol;
pub mod types;

type GlobalTable = HashMap<UniqueId, Type>;
type GlobalTableBuffer = Rc<RefCell<GlobalTable>>;

pub struct SemProg
{
    funcs: Vec<Func>,
    symbols: GlobalTable,
}

pub fn semantic_check(prog: &Prog) -> Result<SemProg, Vec<CompileError>>
{
    // Setup
    let globals_buffer = Rc::new(RefCell::new(GlobalTable::new()));
    let error_buffer = Rc::new(RefCell::new(Errors::new()));

    // Scope Checking
    let new_prog = Scopes::eval_prog(prog, error_buffer.clone(), globals_buffer.clone());

    let globals = globals_buffer.take();

    // Type Checking
    TypeChecker::new(error_buffer.borrow_mut().as_mut(), &globals).check_prog(&new_prog);

    let errors = error_buffer.take();
    if errors.is_empty()
    {
        Ok(SemProg {
            funcs: new_prog.funcs,
            symbols: globals,
        })
    }
    else
    {
        Err(errors)
    }
}
