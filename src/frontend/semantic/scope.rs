use std::collections::HashMap;

use crate::{
    common::ScopeMethods,
    frontend::{
        ErrorBuffer, Ident,
        errors::CompileError,
        parsers::{Prog, expr::Expr, func::Func, stmt::Stmt, types::Type},
        semantic::symbol::{FunctionTypeInfo, SymbolTableBuffer, UniqueId},
    },
};

type Table = HashMap<Ident, UniqueId>;

pub struct Scopes
{
    errors: ErrorBuffer,
    symbols: SymbolTableBuffer,
    parent: Table,
    local: Table,
}

impl Scopes
{
    pub fn new(errors: ErrorBuffer, symbols: SymbolTableBuffer) -> Self
    {
        Scopes {
            errors,
            symbols: symbols,
            parent: Table::new(),
            local: Table::new(),
        }
    }

    pub fn eval_prog(prog: &Prog, errors: ErrorBuffer, globals: SymbolTableBuffer) -> Prog
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
            symbols: parent.symbols.clone(),
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
        let uid = self.symbols.borrow().new_global_id(name);
        self.symbols.borrow_mut().insert_global(uid.clone(), ty);

        uid
    }

    fn new_untyped_symbol(&mut self, name: Ident) -> UniqueId
    {
        let uid = self.symbols.borrow().new_global_id(name);
        self.symbols.borrow_mut().insert_untyped(uid.clone());

        uid
    }

    fn new_var_symbol(&mut self, ty: Option<Type>, name: Ident) -> UniqueId
    {
        let uid = if let Some(t) = ty
        {
            self.new_global_symbol(t, name.clone())
        }
        else
        {
            self.new_untyped_symbol(name.clone())
        };
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
        let Func {
            name,
            parameters,
            return_type,
            block,
        } = func;

        let func_uid = name.clone(); // Function names are already unique
        let ret_type = return_type.clone();

        self.with_scope(|scope| {
            // This could be done lazily but whatever
            let (new_params, param_types) = parameters
                .iter()
                .map(|x| {
                    let (id, ty) = x.clone();
                    scope.can_define_var(&id);

                    let uid = scope.new_var_symbol(Some(ty.clone()), id);
                    ((uid, ty.clone()), ty)
                })
                .unzip();

            let type_info = FunctionTypeInfo {
                params: param_types,
                return_type: ret_type.clone(),
            };

            scope
                .symbols
                .borrow_mut()
                .insert_func(func_uid.clone(), Some(type_info))
                .inspect(|x| {
                    // Declare redeclaration error
                    todo!()
                });

            Func {
                name: func_uid,
                parameters: new_params,
                return_type: ret_type,
                block: scope.check_expr(&block),
            }
        })
    }

    fn check_expr(&mut self, expr: &Expr) -> Expr
    {
        match expr
        {
            Expr::Literal(lit) => Expr::Literal(lit.clone()),
            Expr::Call(id, params) =>
            {
                let (x, y) = self.check_call(id, params);
                Expr::Call(x, y)
            }
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

    fn check_call(&mut self, id: &Ident, params: &Vec<Expr>) -> (UniqueId, Vec<Expr>)
    {
        let uid = id.clone();
        self.symbols.borrow_mut().insert_func(uid.clone(), None);

        (uid, params.iter().map(|x| self.check_expr(x)).collect())
    }

    fn check_ident(&mut self, ident: &Ident) -> Ident
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
                let var_type = ty.as_ref();

                let ex = self.check_expr(rvalue);

                // Check identifier in scope
                self.can_define_var(id);

                let uid = self.new_var_symbol(var_type.cloned(), id.clone());

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
