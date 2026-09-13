use std::collections::HashMap;

use crate::parser::{Atom, Expr, Op, ParseError, Stmt};

type Scope<'src> = HashMap<&'src str, bool>;

pub struct Resolver<'src> {
    scopes: Vec<Scope<'src>>,
}

impl<'src> Resolver<'src> {
    fn new() -> Self {
        Self {
            scopes: vec![
                // Default global scope
                Scope::new(),
            ],
        }
    }

    fn resolve_expr(&self, expr: &Expr<'src>) -> Result<(), ResolveError> {
        match expr {
            Expr::Atom(atom) => match atom {
                Atom::Ident { name, .. } => {
                    // Undefined vars in an expression can only occur when
                    // evaluating an expression of a declared variable.
                    if let Some(defined) = self.cur_scope().get(name)
                        && !defined
                    {
                        return Err(ResolveError::VarReadInItsInitializer);
                    }
                    self.resolve_local(name)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            Expr::Cons(_op, args) => {
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt<'src>) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Expr(expr) => self.resolve_expr(expr)?,
            Stmt::Print(expr) => self.resolve_expr(expr)?,
            Stmt::Var { name, init_val } => {
                self.declare(name);
                if !matches!(init_val, Expr::Atom(Atom::Nil)) {
                    self.resolve_expr(init_val)?;
                }
                self.define(name);
                // self.begin_scope();
            }
            Stmt::Block(statements) => {
                self.begin_scope();
                for stmt in statements {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scope();
            }
            Stmt::If {
                condition,
                stmt,
                else_stmt,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(stmt)?;
                if let Some(else_stmt) = else_stmt {
                    self.resolve_stmt(else_stmt)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    self.resolve_stmt(initializer)?;
                }
                self.resolve_expr(condition)?;
                if let Some(increment) = increment {
                    self.resolve_expr(increment)?;
                }
                self.resolve_stmt(body)?;
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.resolve_expr(expr)?;
                }
            }
            Stmt::FunDecl {
                ident,
                parameters,
                body,
            } => {
                self.declare(ident);
                self.define(ident);
                self.begin_scope();
                for param in parameters {
                    self.declare(param);
                    self.define(param);
                }
                for stmt in body {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scope();
            }
        }
        Ok(())
    }

    fn declare(&mut self, name: &'src str) {
        self.scopes
            .last_mut()
            .expect("At least one scope should be present")
            .insert(name, false);
    }

    fn define(&mut self, name: &'src str) {
        self.scopes
            .last_mut()
            .expect("At least one scope should be present")
            .entry(name)
            // TODO: raise error if defining undeclared var?
            .and_modify(|e| *e = true);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn cur_scope(&self) -> &Scope<'src> {
        self.scopes
            .last()
            .expect("At least one scope should be present")
    }

    fn cur_scope_mut(&mut self) -> &mut Scope<'src> {
        self.scopes
            .last_mut()
            .expect("At least one scope should be present")
    }

    fn resolve_local(&self, ident: &'src str) -> Result<u16, ResolveError> {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.get(ident).is_some() {
                return Ok(distance as u16);
            }
        }
        Err(ResolveError::UndeclaredVar(ident.to_owned()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("Can't read local variable in its own initializer.")]
    VarReadInItsInitializer,
    #[error("Undeclared variable '{0}'")]
    UndeclaredVar(String),
}
