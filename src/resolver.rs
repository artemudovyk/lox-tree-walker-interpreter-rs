use crate::parser::{Atom, Expr, ParseError, Stmt};
use crate::parser::{Loc, Parser};
use std::collections::HashMap;

type Scope<'src> = HashMap<&'src str, bool>;

pub struct Resolver<'src> {
    scopes: Vec<Scope<'src>>,
    pub locals: HashMap<String, usize>,
}

impl<'src> Resolver<'src> {
    pub fn new() -> Self {
        let mut global_scope = Scope::new();
        global_scope.insert("clock", true);

        Self {
            scopes: vec![global_scope],
            locals: HashMap::new(),
        }
    }

    pub fn run(&mut self, src: &'src str) -> Result<(), ResolveError> {
        let mut parser = Parser::new(src);
        while let Some(stmt) = parser.next_stmt()? {
            self.resolve_stmt(&stmt)?;
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr<'src>) -> Result<(), ResolveError> {
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
                    self.resolve_local(name, atom.ident_hash());
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
                // NOTE: commenting it out because of how it breaks things when global functions are
                // involved.
                //
                // for stmt in statements {
                //     // Define function names in the block first to make them available for the
                //     // current scope.
                //     if let &Stmt::FunDecl { ident, .. } = stmt {
                //         self.declare(ident);
                //         self.define(ident);
                //     }
                // }
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
            .entry(name)
            .or_insert(false);
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

    #[allow(dead_code)]
    fn cur_scope_mut(&mut self) -> &mut Scope<'src> {
        self.scopes
            .last_mut()
            .expect("At least one scope should be present")
    }

    fn resolve_local(&mut self, ident: &'src str, hash: String) {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.get(ident).is_some() {
                self.locals.insert(hash, distance);
                return;
            }
        }
    }
}

impl<'src> Default for Resolver<'src> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ResolveError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("Can't read local variable in its own initializer.")]
    VarReadInItsInitializer,
    #[error("Undeclared variable '{0}'")]
    UndeclaredVar(String),
}
