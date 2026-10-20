use crate::parser::Parser;
use crate::parser::{Atom, Expr, ParseError, Stmt};
use std::collections::HashMap;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ResolveError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("Can't read local variable in its own initializer.")]
    VarReadInItsInitializer,
    #[error("Undeclared variable '{0}'")]
    UndeclaredVar(String),
    #[error("Error at '{0}': Already a variable with this name in this scope.")]
    RedeclarationInLocalScope(String),
}

type Scope<'src> = HashMap<&'src str, bool>;

pub struct Resolver<'src> {
    scopes: Vec<Scope<'src>>,
    pub locals: HashMap<String, usize>,
    in_loop: bool,
    parser: Parser<'src>,
}

impl<'src> Resolver<'src> {
    pub fn new() -> Self {
        let mut global_scope = Scope::new();
        global_scope.insert("clock", true);

        Self {
            scopes: vec![global_scope],
            locals: HashMap::new(),
            in_loop: false,
            parser: Parser::new(""),
        }
    }

    pub fn run(&mut self, src: &'src str) -> Result<(), ResolveError> {
        self.parser = Parser::new(src);
        while let Some(stmt) = self.parser.next_stmt()? {
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
                if self.local_in_current_scope(name) {
                    return Err(ResolveError::RedeclarationInLocalScope(name.to_string()));
                }
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
                self.in_loop = true;
                self.resolve_stmt(body)?;
                self.in_loop = false;
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                self.begin_scope();
                if let Some(initializer) = initializer {
                    self.resolve_stmt(initializer)?;
                }
                self.resolve_expr(condition)?;
                if let Some(increment) = increment {
                    self.resolve_expr(increment)?;
                }
                self.in_loop = true;
                self.resolve_stmt(body)?;
                self.in_loop = false;
                self.end_scope();
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
                    if self.local_in_current_scope(param) {
                        return Err(ResolveError::RedeclarationInLocalScope(param.to_string()));
                    }
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

    fn local_in_current_scope(&self, ident: &'src str) -> bool {
        // Global scope is allowed to have multiple redeclarations (lox has a janky approach to
        // global stuff I do not agree with).
        if self.scopes.len() == 1 {
            return false;
        }
        self.cur_scope().contains_key(ident)
    }

    pub fn current_line(&self) -> usize {
        self.parser.current_line()
    }
}

impl<'src> Default for Resolver<'src> {
    fn default() -> Self {
        Self::new()
    }
}
