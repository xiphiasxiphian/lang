use std::{
    collections::HashSet,
    ops::{BitAnd, BitOr},
    sync::LazyLock,
};

use enum_map::{EnumMap, enum_map};
use itertools::Itertools;
use nom::Err;

use crate::{common::ScopeMethods, frontend::{
    errors::CompileError,
    parsers::{
        expr::{BinOpMode, Expr, Literal, UnaryOpMode}, func::Func, lvalue::LValue, stmt::Stmt, types::{BasicType, Type}, Prog
    },
    semantic::symbol::{FunctionTypeInfo, SymbolTable, UniqueId}, Errors,
}};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeCheckError
{
    Error,
    Unknown(Type)
}

type CheckResult = Result<Type, TypeCheckError>;
trait CheckResultMethods
{
    fn resolve(&self, c: &TypeContraint) -> CheckResult;
    fn assert_known<F: FnOnce() -> ()>(self, err: F) -> CheckResult;
}

impl CheckResultMethods for CheckResult
{
    fn resolve(&self, c: &TypeContraint) -> CheckResult
    {
        match (self, c)
        {
            (Err(TypeCheckError::Unknown(Type::Array(_))), TypeContraint::Is(t @ Type::Array(_))) => Ok(t.clone()),
            (Err(TypeCheckError::Unknown(Type::BasicType(_))), TypeContraint::Is(t @ Type::BasicType(_))) => Ok(t.clone()),
            (Err(TypeCheckError::Unknown(_)), TypeContraint::Is(t)) => Ok(t.clone()),
            (a @ Err(TypeCheckError::Unknown(_)), TypeContraint::And(x, y)) => {
                let first = a.resolve(x);
                let second = a.resolve(y);

                match (first, second)
                {
                    (Ok(t1), Ok(t2)) if t1 == t2 => Ok(t1),
                    (Ok(t), _) => Ok(t),
                    (_, Ok(t)) => Ok(t),
                    _ => a.clone()
                }
            }
            (a @ Err(TypeCheckError::Unknown(_)), TypeContraint::Or(x, y)) => {
                let first = a.resolve(x);
                let second = a.resolve(y);

                match (first, second)
                {
                    (Ok(t1), Ok(t2)) if t1 == t2 => Ok(t1),
                    _ => a.clone()
                }
            }
            (x, _) => x.clone(),
        }
    }

    fn assert_known<F: FnOnce() -> ()>(self, err: F) -> CheckResult {
        if let Err(TypeCheckError::Unknown(_)) = self
        {
            err()
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeContraint<'a>
{
    Any,
    Is(Type),
    IsIndexable,
    IsComparibleTo(Type),
    Or(&'a TypeContraint<'a>, &'a TypeContraint<'a>),
    And(&'a TypeContraint<'a>, &'a TypeContraint<'a>),
}

// Largely for convenience
impl<'a> BitAnd for &'a TypeContraint<'a>
{
    type Output = TypeContraint<'a>;
    fn bitand(self, rhs: Self) -> Self::Output
    {
        TypeContraint::And(self, rhs)
    }
}

// Largely for convenience
impl<'a> BitOr for &'a TypeContraint<'a>
{
    type Output = TypeContraint<'a>;
    fn bitor(self, rhs: Self) -> Self::Output
    {
        TypeContraint::Or(self, rhs)
    }
}

static BASIC_TYPE_CONSTRAINTS: LazyLock<EnumMap<BasicType, HashSet<TypeContraint>>> =
    LazyLock::new(|| {
        use BasicType as B;
        use TypeContraint as T;

        enum_map! {
            B::Int => HashSet::from([
                T::IsComparibleTo(Type::BasicType(B::Char)),
            ]),
            B::Char => HashSet::from([
                T::IsComparibleTo(Type::BasicType(B::Int))
            ]),
            B::Bool => HashSet::from([]),
            B::String => HashSet::from([]),
        }
        .map(|k, mut v| {
            v.extend([
                T::Is(Type::BasicType(k.clone())),
                T::IsComparibleTo(Type::BasicType(k.clone())),
            ]);
            v
        })
    });

impl Type
{
    fn satisfies(&self, c: &TypeContraint) -> bool
    {
        match (self, c)
        {
            (_, TypeContraint::Any) => true,
            (t, TypeContraint::Or(c1, c2)) => t.satisfies(c1) || t.satisfies(c2),
            (t, TypeContraint::And(c1, c2)) => t.satisfies(c1) && t.satisfies(c2),
            (t, TypeContraint::IsIndexable) => t.get_indexed_type().is_some(),
            (Type::Void, TypeContraint::Is(Type::Void)) => true,
            (Type::Void, _) => false,
            (Type::BasicType(basic), constr) => BASIC_TYPE_CONSTRAINTS[basic.clone()].contains(constr),
            (Type::Array(i1), TypeContraint::Is(Type::Array(i2))) => i1 == i2,
            (Type::Array(_), _) => false,
        }
    }

    fn satisfies_then(self, c: &TypeContraint) -> Option<Self>
    {
        self.satisfies(c).then_some(self)
    }

    fn get_indexed_type(&self) -> Option<Type>
    {
        match self
        {
            Type::Array(t) => Some(t.as_ref().clone()),
            _ => None,
        }
    }
}

pub struct TypeChecker<'a>
{
    errors: &'a mut Errors,
    symbols: &'a mut SymbolTable,
}

impl<'a> TypeChecker<'a>
{
    pub fn new(errors: &'a mut Errors, symbols: &'a mut SymbolTable) -> Self
    {
        TypeChecker { errors, symbols }
    }

    pub fn check_prog(&mut self, prog: &Prog)
    {
        prog.funcs.iter().for_each(|x| {
            _ = self.check_func(x).assert_known(|| {
                // TODO: Temporary Error
                self.errors.push(CompileError::blank_error());
            });
        });
    }

    fn check_type(&mut self, ty: Type, c: &TypeContraint) -> CheckResult
    {
        ty.clone().satisfies_then(c).ok_or_else(|| {
            // Temporary Error, TODO: Add proper type error
            self.errors.push(CompileError::raw_error(format!(
                "contraint: {c:?} ty: {ty}",
            )));
            TypeCheckError::Error
        })
    }

    fn check_expr(&mut self, expr: &Expr, c: &TypeContraint) -> CheckResult
    {
        match expr
        {
            Expr::Literal(lit) => self.check_literal(lit, c),
            Expr::Array(exs) => self.check_array(exs, c),
            Expr::LValue(lv) => self.check_lvalue(lv, c),
            Expr::Call(id, ps) =>
            {
                let info = self.get_func_info(id);

                // Length Check
                if info.params.len() != ps.len()
                {
                    // TODO: Report Error
                    self.errors.push(CompileError::blank_error());
                }

                for (p, ty) in ps.iter().zip(info.params)
                {
                    _ = self.check_expr(p, &TypeContraint::Is(ty)).assert_known(|| {
                        // TODO: Temporary Error
                        self.errors.push(CompileError::blank_error());
                    });
                }

                self.check_type(info.return_type, c)
            }
            Expr::UnaryOp(mode, p) => self.check_unary_op(mode, p, c),
            Expr::BinaryOp(mode, p1, p2) => self.check_binary_op(mode, p1, p2, c),
            Expr::Stmt(stmt) => self.check_stmt(stmt, c),
            Expr::Block(exs, ret) =>
            {
                for ele in exs
                {
                    _ = self.check_expr(ele, &TypeContraint::Any).assert_known(|| {
                        // TODO: Temporary Error
                        self.errors.push(CompileError::blank_error());
                    });
                }

                ret.as_ref()
                    .map(|x| self.check_expr(x.as_ref(), c))
                    .unwrap_or_else(|| self.check_type(Type::Void, c))
            }
        }
    }

    fn check_func(&mut self, func: &Func) -> CheckResult
    {
        self.check_expr(&func.block, &TypeContraint::Is(func.return_type.clone()))
    }

    fn check_literal(&mut self, lit: &Literal, c: &TypeContraint) -> CheckResult
    {
        self.check_type(
            match lit
            {
                Literal::Int(_) => Type::BasicType(BasicType::Int),
                Literal::Char(_) => Type::BasicType(BasicType::Char),
                Literal::Bool(_) => Type::BasicType(BasicType::Bool),
                Literal::String(_) => Type::BasicType(BasicType::String),
            },
            c,
        )
    }

    fn check_lvalue(&mut self, lvalue: &LValue, c: &TypeContraint) -> CheckResult
    {
        match lvalue
        {
            LValue::Ident(id) => {
                let ty = self
                    .symbols
                    .get_global(id)
                    .cloned()
                    .expect("Failed to get expected global while type checking");

                self.check_type(ty, c)
            }
            LValue::ArrayElem(lv, ex) => {
                let lv_typed = self.check_lvalue(lv, &TypeContraint::IsIndexable)?;
                _ = self.check_expr(ex, &TypeContraint::Is(Type::BasicType(BasicType::Int)))?;

                self.check_type(lv_typed.get_indexed_type().expect("Type wasnt indexable, which should have been filtered"), c)
            }
        }
    }

    fn check_array(&mut self, exprs: &Vec<Expr>, c: &TypeContraint) -> CheckResult
    {
        if exprs.is_empty() { Err(TypeCheckError::Unknown(Type::Array(Box::new(Type::Void)))).resolve(c) }
        else
        {
            let iter = exprs.iter().map(|x| self.check_expr(x, &TypeContraint::Any)).unique().collect_vec();
            let array_type = Type::Array(Box::new(
                self.fold_types(iter.into_iter())?
            ));

            self.check_type(array_type, c)
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt, c: &TypeContraint) -> CheckResult
    {
        match stmt
        {
            Stmt::If { cond, tt, ff } =>
            {
                let cond_typed =
                    self.check_expr(cond, &TypeContraint::Is(Type::BasicType(BasicType::Bool)));
                let tt_typed = self.check_expr(tt, c);

                let body_typed = ff
                    .as_ref()
                    .map(|el| self.check_expr(el, &TypeContraint::Is(tt_typed?)))
                    .unwrap_or_else(|| self.check_type(Type::Void, c));

                cond_typed.and(body_typed)
            }
            Stmt::Assign { lv, rvalue } => {
                match lv
                {
                    LValue::Ident(uid) => {
                        self.with_symbol(uid, |checker, x| {
                            checker.check_expr(
                                rvalue.as_ref(),
                                &(&(x
                                    .map(|t| TypeContraint::Is(t))
                                    .unwrap_or(TypeContraint::Any))
                                    & c),
                            )
                        })
                    },
                    a @ LValue::ArrayElem(_, ex) => {
                        _ = self.check_expr(ex, &TypeContraint::Is(Type::BasicType(BasicType::Int)));
                        let lv_typed = self.check_lvalue(a, &TypeContraint::Any)?;

                        self.check_expr(rvalue, &TypeContraint::Is(lv_typed))
                    }
                }
            },
            Stmt::While { cond, then } =>
            {
                let cond_typed =
                    self.check_expr(cond, &TypeContraint::Is(Type::BasicType(BasicType::Bool)));
                let body_typed = self.check_expr(then.as_ref(), c);

                cond_typed.and(body_typed)
            }
            Stmt::Declare { .. } => unreachable!(), // Declares should have been filtered by scope checking
        }
    }

    fn check_unary_op(&mut self, mode: &UnaryOpMode, p: &Expr, c: &TypeContraint) -> CheckResult
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

        self.check_expr(p, &mode_info.0)
            .and_then(|_| self.check_type(mode_info.1, c))
    }

    fn check_binary_op(
        &mut self,
        mode: &BinOpMode,
        p1: &Expr,
        p2: &Expr,
        c: &TypeContraint,
    ) -> CheckResult
    {
        use BinOpMode as B;

        // Kinda temporary as won't work with operators being used for multiple different types.
        let mode_info = match mode
        {
            B::Add | B::Sub | B::Mul | B::Div => (
                TypeContraint::Is(Type::BasicType(BasicType::Int)),
                TypeContraint::Is(Type::BasicType(BasicType::Int)),
                Type::BasicType(BasicType::Int),
            ),
        };

        self.check_expr(p1, &mode_info.0)
            .and(self.check_expr(p2, &mode_info.1))
            .and_then(|_| self.check_type(mode_info.2, c))
    }

    fn with_symbol<F>(&mut self, symbol: &UniqueId, f: F) -> CheckResult
    where
        F: FnOnce(&mut Self, Option<Type>) -> CheckResult,
    {
        let res = self.symbols.get_global(symbol).cloned();

        f(self, res).inspect(|x| self.symbols.set_untyped(symbol.clone(), x.clone()))
    }

    fn get_func_info(&self, id: &UniqueId) -> FunctionTypeInfo
    {
        self.symbols
            .get_func_info(id)
            .cloned()
            .expect(format!("Id {id} not found. Did Scope Checking Succeed properly").as_str())
    }

    fn fold_types<T>(&mut self, it: T) -> CheckResult
    where
        T: Iterator<Item = CheckResult>,
    {
        it.reduce(|x, y| {
            match (x, y)
            {
                (a @ Err(TypeCheckError::Error), _) => a,
                (_, a @ Err(TypeCheckError::Error)) => a,
                (u @ Err(TypeCheckError::Unknown(_)), Ok(t)) => u.resolve(&TypeContraint::Is(t)),
                (Ok(t), u @ Err(TypeCheckError::Unknown(_))) => u.resolve(&TypeContraint::Is(t)),
                (Err(TypeCheckError::Unknown(u1)), Err(TypeCheckError::Unknown(u2))) =>
                    Err(Self::unify_unknowns(u1, u2).map(|x| TypeCheckError::Unknown(x)).unwrap_or(TypeCheckError::Error)),
                (Ok(t1), Ok(t2)) => Self::unify_types(t1, t2).ok_or(TypeCheckError::Error),
            }
        }).unwrap_or_else(|| Err(TypeCheckError::Unknown(Type::Void)))
        .also(|x| {
            // Report Error. TODO: Temporary Error
            if let Err(TypeCheckError::Error) = x { self.errors.push(CompileError::blank_error()); }
        })
    }

    fn unify_types(t1: Type, t2: Type) -> Option<Type>
    {
        // This will be more complicated when more complex type interactions are introduced
        (t1 == t2).then(|| t1)
    }

    fn unify_unknowns(u1: Type, u2: Type) -> Option<Type>
    {
        match (u1, u2)
        {
            (a @ Type::Array(_), Type::Array(_)) => Some(a),
            (a @ Type::BasicType(_), Type::BasicType(_)) => Some(a),
            (Type::Void, a) => Some(a),
            (a, Type::Void) => Some(a),
            _ => None,
        }
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
        let mut errors = Errors::new();

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

        assert!(matches!(
            checker.check_expr(&Expr::Stmt(good_if_stmt), &TypeContraint::Any),
            Ok(Type::BasicType(BasicType::Int))
        ));
        assert!(matches!(
            checker.check_expr(&Expr::Stmt(bad_if_stmt), &TypeContraint::Any),
            Err(_)
        ));
        assert!(matches!(
            checker.check_expr(&Expr::Stmt(void_if_stmt), &TypeContraint::Any),
            Ok(Type::Void)
        ));
        assert!(matches!(
            checker.check_expr(&Expr::Stmt(bad_cond_if_stmt), &TypeContraint::Any),
            Err(_)
        ));
    }

    #[test]
    #[ignore = "doesnt work without scope checking first"]
    fn assign_check()
    {
        let mut table = SymbolTable::new();
        let mut errors = Errors::new();

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

        assert!(matches!(
            checker.check_expr(&good_assign, &TypeContraint::Any),
            Ok(Type::BasicType(BasicType::Int))
        ));
        assert!(matches!(checker.check_expr(&bad_assign, &TypeContraint::Any), Err(_)));
    }
}
