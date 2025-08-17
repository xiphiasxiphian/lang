use std::{
    collections::HashSet,
    ops::{BitAnd, BitOr},
    str::FromStr,
    sync::LazyLock,
};

use enum_map::{EnumMap, enum_map};

use crate::frontend::{
    parsers::{
        expr::{BinOpMode, Expr, Literal, UnaryOpMode},
        func::Func,
        stmt::Stmt,
        types::{BasicType, Type},
    },
    semantic::symbol::SymbolTable,
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

struct TypeChecker<'a>
{
    symbols: &'a SymbolTable,
}

impl<'a> TypeChecker<'a>
{
    pub fn new(symbols: &'a SymbolTable) -> Self
    {
        TypeChecker { symbols }
    }

    pub fn check_expr(&self, expr: &Expr, c: TypeContraint) -> Option<Type>
    {
        match expr
        {
            Expr::Literal(lit) => self.check_literal(lit, c),
            Expr::Ident(id) =>
            {
                // Need to figure out unique id stuff for this to actually work ...
                let entry = &String::from_str(id).expect("Failed to parse id string");

                self.symbols.get(id).satisfies_then(c)
            }
            Expr::Call(id, ps) => todo!(),
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

    pub fn check_func(&self, func: &Func, c: TypeContraint) -> Option<Type>
    {
        let ret_expr_typed =
            self.check_expr(&func.block, c & TypeContraint::Is(func.return_type.clone()));
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

    fn check_stmt(&self, stmt: &Stmt, c: TypeContraint) -> Option<Type>
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
                self.check_expr(rvalue.as_ref(), TypeContraint::Is(self.symbols.get(id)) & c)
            }
            Stmt::While { cond, then } =>
            {
                let cond_typed =
                    self.check_expr(cond, TypeContraint::Is(Type::BasicType(BasicType::Bool)));
                let body_typed = self.check_expr(then.as_ref(), c);

                cond_typed.and(body_typed)
            }
            _ => unreachable!(),
        }
    }

    fn check_unary_op(&self, mode: &UnaryOpMode, p: &Expr, c: TypeContraint) -> Option<Type>
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
        &self,
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
}

#[cfg(test)]
mod type_check_tests
{
    use super::*;

    #[test]
    fn if_stmt_check()
    {
        let table = SymbolTable::new();
        let checker = TypeChecker::new(&table);

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
    fn assign_check()
    {
        let table = SymbolTable::new();
        let checker = TypeChecker::new(&table);

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
