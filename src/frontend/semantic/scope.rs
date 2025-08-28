use std::collections::HashMap;

use crate::{
    common::ScopeMethods,
    frontend::{
        ErrorBuffer, Ident,
        errors::CompileError,
        parsers::{Prog, expr::Expr, func::Func, stmt::Stmt, types::Type},
        semantic::{
            GlobalTableBuffer,
            symbol::{UniqueId, gen_id},
        },
    },
};

#[derive(Clone)]
struct FunctionTypeInfo
{
    params: Vec<Type>,
    return_type: Type,
}

type FunctionTable = HashMap<UniqueId, FunctionTypeInfo>;
type Table = HashMap<Ident, UniqueId>;

pub struct Scopes
{
    errors: ErrorBuffer,
    functions: FunctionTable,
    global: GlobalTableBuffer,
    parent: Table,
    local: Table,
}

impl Scopes
{
    pub fn new(errors: ErrorBuffer, globals: GlobalTableBuffer) -> Self
    {
        Scopes {
            errors,
            functions: FunctionTable::new(),
            global: globals,
            parent: Table::new(),
            local: Table::new(),
        }
    }

    pub fn eval_prog(prog: &Prog, errors: ErrorBuffer, globals: GlobalTableBuffer) -> Prog
    {
        let mut scopes = Self::new(errors, globals);
        let new_prog = scopes.check_prog(prog);

        new_prog
    }

    fn check_prog(&mut self, prog: &Prog) -> Prog
    {
        Prog {
            funcs: prog.funcs.iter().map(|x| self.check_func(x)).collect(),
        }
    }

    fn from_parent(parent: &Self) -> Self
    {
        let new_parent = parent
            .parent
            .clone()
            .also_mut(|x| x.extend(parent.local.clone()));

        Scopes {
            errors: parent.errors.clone(),
            functions: parent.functions.clone(),
            global: parent.global.clone(),
            parent: new_parent,
            local: HashMap::new(),
        }
    }

    fn with_scope<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Scopes) -> T,
    {
        let mut child_scope = Scopes::from_parent(self);
        f(&mut child_scope)
    }

    fn new_global_symbol(&mut self, ty: Type, name: Ident) -> UniqueId
    {
        let uid = gen_id(name, self.global.borrow().len());
        self.global.borrow_mut().insert(uid.clone(), ty);

        uid
    }

    fn new_var_symbol(&mut self, ty: Type, name: Ident) -> UniqueId
    {
        let uid = self.new_global_symbol(ty, name.clone());
        self.local.insert(name, uid.clone());

        uid
    }

    fn get_var(&mut self, name: &Ident) -> UniqueId
    {
        self.local.get(name).cloned().unwrap_or_else(|| {
            self.parent.get(name).cloned().unwrap_or_else(|| {
                self.errors.borrow_mut().push(CompileError::blank_error());
                self.new_global_symbol(Type::Void, name.clone())
            })
        })
    }

    fn can_define_var(&mut self, name: &Ident)
    {
        if let Some(x) = self.local.get(name)
        {
            // Define Error Here
            /* TEMPORARY */
            self.errors.borrow_mut().push(CompileError::blank_error());
        }
    }

    // Scope Checking
    fn check_block(&mut self, body: &Vec<Expr>, ret: Option<&Expr>) -> Expr
    {
        self.with_scope(|child| {
            Expr::Block(
                body.iter().map(|x| child.check_expr(x)).collect(),
                ret.map(|x| Box::new(child.check_expr(x))),
            )
        })
    }

    fn check_func(&mut self, func: &Func) -> Func
    {
        Func {
            name: func.name.clone(),
            parameters: func.parameters.clone(),
            return_type: func.return_type.clone(),
            block: self.check_expr(&func.block),
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Expr
    {
        match expr
        {
            Expr::Literal(lit) => Expr::Literal(lit.clone()),
            Expr::Call(id, body) => todo!(),
            Expr::Ident(id) => Expr::Ident(self.check_ident(id)),
            Expr::UnaryOp(mode, e) => Expr::UnaryOp(mode.clone(), Box::new(self.check_expr(e))),
            Expr::BinaryOp(mode, e1, e2) => Expr::BinaryOp(
                mode.clone(),
                Box::new(self.check_expr(e1)),
                Box::new(self.check_expr(e2)),
            ),
            Expr::Stmt(stmt) => Expr::Stmt(self.check_stmt(stmt)),
            Expr::Block(exs, ret) => self.check_block(exs, ret.as_ref().map(|x| x.as_ref())),
        }
    }

    fn check_ident(&mut self, ident: &String) -> Ident
    {
        self.get_var(ident)
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Stmt
    {
        match stmt
        {
            Stmt::If { cond, tt, ff } => Stmt::If {
                cond: Box::new(self.check_expr(cond)),
                tt: Box::new(self.check_expr(tt)),
                ff: ff.as_ref().map(|x| Box::new(self.check_expr(x))),
            },
            Stmt::Declare { id, ty, rvalue } =>
            {
                let var_type = ty.as_ref().unwrap_or(todo!()); // TODO: Need to find nice way of working out implicit typing

                // Check identifier in scope
                self.can_define_var(id);

                let ex = self.check_expr(rvalue);
                let uid = self.new_var_symbol(*var_type, *id);

                Stmt::Assign {
                    id: uid,
                    rvalue: Box::new(ex),
                }
            }
            Stmt::Assign { id, rvalue } => Stmt::Assign {
                id: self.check_ident(id),
                rvalue: Box::new(self.check_expr(rvalue)),
            },
            Stmt::While { cond, then } => Stmt::While {
                cond: Box::new(self.check_expr(cond)),
                then: Box::new(self.check_expr(then)),
            },
        }
    }
}

#[cfg(test)]
mod scope_tests
{
    use super::*;
    use crate::frontend::parsers::expr::UnaryOpMode;
}
