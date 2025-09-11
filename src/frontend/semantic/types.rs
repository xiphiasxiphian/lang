use std::{
    collections::HashSet,
    ops::{BitAnd, BitOr},
    str::FromStr,
    sync::LazyLock,
};

use enum_map::{EnumMap, enum_map};

use crate::frontend::{
    errors::CompileError,
    parsers::{
        expr::{BinOpMode, Expr, Literal, UnaryOpMode}, func::Func, stmt::Stmt, types::{BasicType, Type}, Prog
    },
    semantic::symbol::{FunctionTypeInfo, SymbolTable, UniqueId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeContraint
{
    Any,
    Is(Type),
    IsComparibleTo(Type),
    Or(Box<TypeContraint>, Box<TypeContraint>),
    And(Box<TypeContraint>, Box<TypeContraint>),
}

// Largely for convenience
impl BitAnd for TypeContraint
{
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output
    {
        TypeContraint::And(Box::new(self), Box::new(rhs))
    }
}

// Largely for convenience
impl BitOr for TypeContraint
{
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output
    {
        TypeContraint::Or(Box::new(self), Box::new(rhs))
    }
}

static BASIC_TYPE_CONSTRAINTS: LazyLock<EnumMap<BasicType, HashSet<TypeContraint>>> =
    LazyLock::new(|| {
        use BasicType::*;
        use TypeContraint::*;

        enum_map! {
            Int => HashSet::from([
                IsComparibleTo(Type::BasicType(Char)),
            ]),
            Char => HashSet::from([
                IsComparibleTo(Type::BasicType(Int))
            ]),
            Bool => HashSet::from([]),
            String => HashSet::from([]),
        }
        .map(|k, mut v| {
            v.extend([
                Is(Type::BasicType(k.clone())),
                IsComparibleTo(Type::BasicType(k.clone())),
            ]);
            v
        })
    });

impl Type
{
    fn satisfies(&self, c: TypeContraint) -> bool
    {
        match (self, c)
        {
            (_, TypeContraint::Any) => true,
            (t, TypeContraint::Or(c1, c2)) => t.satisfies(*c1) || t.satisfies(*c2),
            (t, TypeContraint::And(c1, c2)) => t.satisfies(*c1) && t.satisfies(*c2),
            (Type::Void, TypeContraint::Is(Type::Void)) => true,
            (Type::Void, _) => false,
            (Type::BasicType(basic), constr) =>
            {
                BASIC_TYPE_CONSTRAINTS[basic.clone()].contains(&constr)
            }
            (Type::Array(inner), _) => todo!(),
        }
    }

    fn satisfies_then(self, c: TypeContraint) -> Option<Self>
    {
        if self.satisfies(c) { Some(self) } else { None }
    }
}

pub struct TypeChecker<'a>
{
    errors: &'a mut Vec<CompileError>,
    symbols: &'a mut SymbolTable,
}

impl<'a> TypeChecker<'a>
{
    pub fn new(errors: &'a mut Vec<CompileError>, symbols: &'a mut SymbolTable) -> Self
    {
        TypeChecker { errors, symbols }
    }

    pub fn check_prog(&mut self, prog: &Prog)
    {
        prog.funcs.iter().for_each(|x| {self.check_func(x);});
    }

    fn check_expr(&mut self, expr: &Expr, c: TypeContraint) -> Option<Type>
    {
        match expr
        {
            Expr::Literal(lit) => self.check_literal(lit, c),
            Expr::Ident(id) => self.symbols.get_global(id)
                .cloned()
                .expect("Failed to get expected global while type checking")
                .satisfies_then(c),
            Expr::Call(id, ps) => {
                let info = self.get_func_info(id);

                // Length Check
                if info.params.len() != ps.len() {
                    // Report Error
                }

                for (p, ty) in ps.iter().zip(info.params) {
                    self.check_expr(p, TypeContraint::Is(ty));
                }

                info.return_type.satisfies_then(c)
            }
            Expr::UnaryOp(mode, p) => self.check_unary_op(mode, p, c),
            Expr::BinaryOp(mode, p1, p2) => self.check_binary_op(mode, p1, p2, c),
            Expr::Stmt(stmt) => self.check_stmt(stmt, c),
            Expr::Block(exs, ret) =>
            {
                for ele in exs
                {
                    self.check_expr(ele, TypeContraint::Any);
                }

                ret.as_ref()
                    .map(|x| self.check_expr(x.as_ref(), c.clone()))
                    .unwrap_or(Type::Void.satisfies_then(c))
            }
        }
    }

    fn check_func(&mut self, func: &Func) -> Option<Type>
    {
        let ret_expr_typed =
            self.check_expr(&func.block, TypeContraint::Is(func.return_type.clone()));
        ret_expr_typed.filter(|t| *t == func.return_type)
    }

    fn check_literal(&self, lit: &Literal, c: TypeContraint) -> Option<Type>
    {
        match lit
        {
            Literal::Int(_) => Type::BasicType(BasicType::Int),
            Literal::Char(_) => Type::BasicType(BasicType::Char),
            Literal::Bool(_) => Type::BasicType(BasicType::Bool),
            Literal::String(_) => Type::BasicType(BasicType::String),
        }
        .satisfies_then(c)
    }

    fn check_stmt(&mut self, stmt: &Stmt, c: TypeContraint) -> Option<Type>
    {
        match stmt
        {
            Stmt::If { cond, tt, ff } =>
            {
                let cond_typed =
                    self.check_expr(cond, TypeContraint::Is(Type::BasicType(BasicType::Bool)));
                let tt_typed = self.check_expr(tt, c.clone());

                let body_typed = ff
                    .as_ref()
                    .map(|el| self.check_expr(el, TypeContraint::Is(tt_typed?)))
                    .unwrap_or(Type::Void.satisfies_then(c));

                cond_typed.and(body_typed)
            }
            Stmt::Assign { id, rvalue } =>
            {
                self.with_symbol(
                    id,
                    |checker, x| checker.check_expr(
                        rvalue.as_ref(),
                        x
                            .map(|t| TypeContraint::Is(t))
                            .unwrap_or(TypeContraint::Any) & c
                    )
                )
            }
            Stmt::While { cond, then } =>
            {
                let cond_typed =
                    self.check_expr(cond, TypeContraint::Is(Type::BasicType(BasicType::Bool)));
                let body_typed = self.check_expr(then.as_ref(), c);

                cond_typed.and(body_typed)
            }
            _ => unreachable!(), // Declares should have been filtered by scope checking
        }
    }

    fn check_unary_op(&mut self, mode: &UnaryOpMode, p: &Expr, c: TypeContraint) -> Option<Type>
    {
        let mode_info = match mode
        {
            UnaryOpMode::Neg => (
                TypeContraint::Is(Type::BasicType(BasicType::Int)),
                Type::BasicType(BasicType::Int),
            ),
            UnaryOpMode::Not => (
                TypeContraint::Is(Type::BasicType(BasicType::Bool)),
                Type::BasicType(BasicType::Bool),
            ),
        };

        self.check_expr(p, mode_info.0)
            .and_then(|_| mode_info.1.satisfies_then(c))
    }

    fn check_binary_op(
        &mut self,
        mode: &BinOpMode,
        p1: &Expr,
        p2: &Expr,
        c: TypeContraint,
    ) -> Option<Type>
    {
        use BinOpMode::*;

        // Kinda temporary as won't work with operators being used for multiple different types.
        let mode_info = match mode
        {
            Add | Sub | Mul | Div => (
                TypeContraint::Is(Type::BasicType(BasicType::Int)),
                TypeContraint::Is(Type::BasicType(BasicType::Int)),
                Type::BasicType(BasicType::Int),
            ),
        };

        self.check_expr(p1, mode_info.0)
            .and(self.check_expr(p2, mode_info.1))
            .and_then(|_| mode_info.2.satisfies_then(c))
    }

    fn with_symbol<F>(&mut self, symbol: &UniqueId, f: F) -> Option<Type>
    where F: FnOnce(&mut Self, Option<Type>) -> Option<Type>
    {
        let res = self.symbols
                    .get_global(symbol)
                    .cloned();

        f(
            self,
            res
        ).inspect(|x| self.symbols.set_untyped(symbol.clone(), x.clone()))
    }

    fn get_func_info(&self, id: &UniqueId) -> FunctionTypeInfo
    {
        self.symbols
            .get_func_info(id)
            .cloned()
            .expect(format!("Id {id} not found. Did Scope Checking Succeed properly").as_str())
    }
}

#[cfg(test)]
mod type_check_tests
{
    use super::*;

    #[test]
    fn if_stmt_check()
    {
        let mut table = SymbolTable::new();
        let mut errors = Vec::new();

        let mut checker = TypeChecker::new(&mut errors, &mut table);

        let good_if_stmt = Stmt::If {
            cond: Box::new(Expr::Literal(Literal::Bool(true))),
            tt: Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Int(32)))),
            )),
            ff: Some(Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Int(42)))),
            ))),
        };

        let bad_if_stmt = Stmt::If {
            cond: Box::new(Expr::Literal(Literal::Bool(true))),
            tt: Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Int(32)))),
            )),
            ff: Some(Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Char('a')))),
            ))),
        };

        let void_if_stmt = Stmt::If {
            cond: Box::new(Expr::Literal(Literal::Bool(true))),
            tt: Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Int(32)))),
            )),
            ff: None,
        };

        let bad_cond_if_stmt = Stmt::If {
            cond: Box::new(Expr::Literal(Literal::Int(42))),
            tt: Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Int(32)))),
            )),
            ff: Some(Box::new(Expr::Block(
                vec![],
                Some(Box::new(Expr::Literal(Literal::Char('a')))),
            ))),
        };

        assert_eq!(
            checker.check_expr(&Expr::Stmt(good_if_stmt), TypeContraint::Any),
            Some(Type::BasicType(BasicType::Int))
        );
        assert_eq!(
            checker.check_expr(&Expr::Stmt(bad_if_stmt), TypeContraint::Any),
            None
        );
        assert_eq!(
            checker.check_expr(&Expr::Stmt(void_if_stmt), TypeContraint::Any),
            Some(Type::Void)
        );
        assert_eq!(
            checker.check_expr(&Expr::Stmt(bad_cond_if_stmt), TypeContraint::Any),
            None
        )
    }

    #[test]
    #[ignore = "doesnt work without scope checking first"]
    fn assign_check()
    {
        let mut table = SymbolTable::new();
        let mut errors = Vec::new();

        let mut checker = TypeChecker::new(&mut errors, &mut table);

        let good_assign = Expr::Stmt(Stmt::Declare {
            id: "a".into(),
            ty: Some(Type::BasicType(BasicType::Int)),
            rvalue: Box::new(Expr::Literal(Literal::Int(42))),
        });

        let bad_assign = Expr::Stmt(Stmt::Declare {
            id: "a".into(),
            ty: Some(Type::BasicType(BasicType::Int)),
            rvalue: Box::new(Expr::Literal(Literal::Char('a'))),
        });

        assert_eq!(
            checker.check_expr(&good_assign, TypeContraint::Any),
            Some(Type::BasicType(BasicType::Int))
        );
        assert_eq!(checker.check_expr(&bad_assign, TypeContraint::Any), None);
    }
}
