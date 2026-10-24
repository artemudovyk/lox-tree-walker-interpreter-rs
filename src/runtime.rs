use crate::environment::Env;
use crate::parser::{Atom, Expr, Op, ParseError, Parser, Stmt};
use crate::resolver::{ResolveError, Resolver};
use std::collections::HashMap;
use std::{
    borrow::Cow,
    cell::RefCell,
    io::Write,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum RuntimeError {
    #[error("{0}")]
    TypeMismatch(String),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    ResolveError(#[from] ResolveError),
    #[error("variable '{0}' is undeclared")]
    UndeclaredVar(String),
    #[error("property '{0}' is undefined")]
    UndefinedProperty(String),
    #[error("Can only call functions and classes.")]
    UncallableType,
    #[error(
        "Function '{func}' called with unexpected number of arguments. Expected {expected_args_num} arguments, but got {got_args_num}."
    )]
    UnexpectedArgsNum {
        func: String,
        expected_args_num: u16,
        got_args_num: u16,
    },
}

pub struct Runtime<'src, W: Write> {
    out: W,
    env: Rc<RefCell<Env<'src>>>,
    parser: Parser<'src>,
    resolver: Resolver<'src>,
}

impl<'src> Runtime<'src, std::io::Stdout> {
    pub fn new() -> Self {
        let mut env = Env::new();
        env.set("clock", Value::NativeFunc(NativeFunc::Clock));

        Self {
            out: std::io::stdout(),
            env: Rc::new(RefCell::new(env)),
            parser: Parser::new(""),
            resolver: Resolver::new(),
        }
    }
}

impl<'src, W: Write> Runtime<'src, W> {
    pub fn with_out(writer: W) -> Self {
        let mut env = Env::new();
        env.set("clock", Value::NativeFunc(NativeFunc::Clock));

        Self {
            out: writer,
            env: Rc::new(RefCell::new(env)),
            parser: Parser::new(""),
            resolver: Resolver::new(),
        }
    }

    pub fn run(&mut self, src: &'src str) -> Result<(), RuntimeError> {
        self.parser = Parser::new(src);
        self.resolver.run(src)?;
        while let Some(stmt) = self.parser.next_stmt()? {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn eval_expr(&mut self, expr: &Expr<'src>) -> Result<Value<'src>, RuntimeError> {
        match expr {
            Expr::Atom(atom) => match atom {
                Atom::Number(n) => Ok(Value::Number(*n)),
                Atom::String(s) => Ok(Value::String(Cow::Borrowed(s))),
                Atom::Bool(b) => Ok(Value::Bool(*b)),
                Atom::Nil => Ok(Value::Nil),
                // Atom::Ident{name, ..} => Ok(self
                //     .env
                //     .borrow()
                //     .get(name)
                //     .map(|v| v.to_owned())
                //     .ok_or(RuntimeError::UndeclaredVar(name.to_string()))?),
                Atom::Ident { name, .. } => Ok(self.look_up_ident(atom)
                     .ok_or(RuntimeError::UndeclaredVar(name.to_string()))?),
            },
            Expr::Cons(op, args) => match op {
                Op::Add => match args.as_slice() {
                    [left, right] => match (dbg!(self.eval_expr(left)?), dbg!(self.eval_expr(right)?)) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => {
                            Ok(Value::String(Cow::Owned(format!("{l}{r}"))))
                        }
                        _ => Err(RuntimeError::TypeMismatch(
                            "Operands must be numbers.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Operands must be numbers.".to_owned(),
                    ))?,
                },
                Op::Group => match args.as_slice() {
                    [arg] => self.eval_expr(arg),
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Minus => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Operands must be numbers.".to_owned(),
                        ))?,
                    },
                    [arg] => match self.eval_expr(arg)? {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Operand must be a number".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Operands must be numbers.".to_owned(),
                    ))?,
                },
                Op::Div => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Operands must be numbers.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Operands must be numbers.".to_owned(),
                    ))?,
                },
                Op::Mult => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Operands must be numbers.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Operands must be numbers.".to_owned(),
                    ))?,
                },
                Op::Bang => match args.as_slice() {
                    [arg] => match self.eval_expr(arg)? {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        Value::Number(n) => Ok(Value::Bool(n == 0.0)),
                        Value::Nil => Ok(Value::Bool(true)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Less => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::LessEq => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Greater => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::GreaterEq => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::EqEq => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l == r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::Bool(l == r)),
                        (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l == r)),
                        _ => Ok(Value::Bool(false)),
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::NotEq => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l != r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::Bool(l != r)),
                        (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l != r)),
                        _ => Ok(Value::Bool(true)),
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Assign => match args.as_slice() {
                    [ident, right] => match (ident, self.eval_expr(right)?) {
                        (Expr::Atom(atom @ Atom::Ident {..}), val) => {
                            // self.env.borrow_mut().update(name, val.clone())?; 
                            // Ok(val)
                            self.update_ident(atom, val.clone())?;
                            Ok(val)
                        }
                        (Expr::Cons(Op::Dot, dot_args), val) => match dot_args.as_slice() {
                            [obj @ Expr::Atom(Atom::Ident { name: obj_name, .. }), Expr::Atom(Atom::Ident { name: property_name, .. })] => match self.eval_expr(obj)? {
                                Value::ClassInstance(instance) => {
                                    instance.borrow_mut().set(property_name, val);
                                    self.env.borrow_mut().set(obj_name, Value::ClassInstance(instance));
                                    Ok(Value::Nil)
                                }
                                _ => Err(RuntimeError::TypeMismatch(
                                    "Unsupported operand types for this operation.".to_owned(),
                                ))?,
                            }
                            _ => Err(RuntimeError::TypeMismatch(
                                "Unsupported operand types for this operation.".to_owned(),
                            ))?,
                        }
                        _ => Err(RuntimeError::TypeMismatch("Assignment must be done on the variable and have an expression on the right side".to_owned()))
                    },
                    _ => Err(RuntimeError::TypeMismatch("Assignment must be done on the variable and have an expression on the right side".to_owned()))
                }
                Op::And => match args.as_slice() {
                    [left, right] => {
                        let left = self.eval_expr(left)?;
                        if !left.is_truthy() {
                            return Ok(left)
                        }
                        let right = self.eval_expr(right)?;
                        Ok(right)
                    }
                    _ => Err(RuntimeError::TypeMismatch("Unsupported operands number for this operation.".to_owned()))?
                }
                Op::Or => match args.as_slice() {
                    [left, right] => {
                        let left = self.eval_expr(left)?;
                        if left.is_truthy() {
                            return Ok(left)
                        }
                        let right = self.eval_expr(right)?;
                        Ok(right)
                    }
                    _ => Err(RuntimeError::TypeMismatch("Unsupported operands number for this operation.".to_owned()))?
                    }
                Op::Dot => match args.as_slice() {
                    [obj, Expr::Atom(Atom::Ident { name: property, .. })] => {
                        match self.eval_expr(obj)? {
                            Value::ClassInstance(instance) => {
                                let p = instance.borrow().get(property)?;
                                if let Value::Func(mut func) = p {
                                    let mut env = func.closure.borrow().clone();
                                    // Do not overwrite `this` in case of class method reassigned
                                    // with method from another class
                                    if env.get("this").is_none() {
                                        env.set("this", Value::ClassInstance(instance));
                                    }
                                    func.closure = Rc::new(RefCell::new(env));
                                    return Ok(Value::Func(func))
                                }
                                Ok(p)
                            }
                            _ => Err(RuntimeError::TypeMismatch("Unsupported operands number for this operation.".to_owned()))?
                        }
                        
                    }
                    _ => Err(RuntimeError::TypeMismatch("Unsupported operands number for this operation.".to_owned()))?
                }
                Op::Call => match args.as_slice() {
                    [callee, args @ ..] => match self.eval_expr(callee)? {
                        Value::NativeFunc(NativeFunc::Clock) => {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards").as_secs_f64();
                            Ok(Value::Number(now))
                        },
                        Value::Func(func) => {
                            let prev_env = self.env.clone();

                            if args.len() != func.parameters.len() {
                                return Err(RuntimeError::UnexpectedArgsNum {
                                    func: func.ident.to_owned(),
                                    expected_args_num: func.parameters.len() as u16,
                                        got_args_num: args.len() as u16,
                                });
                            }
                            // Evaluate all expressions first before swapping the environment to
                            // properly handle recursive calls where the var from the current env
                            // could be used as an argument for a nested call. If we'd swap
                            // environment before evaluating all args to the final value, it would
                            // lead to unexpected calculations.
                            let args = args.iter().map(|arg| self.eval_expr(arg)).collect::<Result<Vec<_>, _>>()?;
                            self.env = Rc::new(RefCell::new(Env::from_closure(func.closure.clone())));
                            for (param, arg) in func.parameters.iter().zip(args) {
                                self.env.borrow_mut().set(param, arg);
                            }

                            let mut val = None;
                            for stmt in func.body {
                                val = self.exec_stmt(stmt)?;
                                if val.is_some() {
                                    break;
                                }
                            }

                            self.env = prev_env;

                            Ok(val.unwrap_or(Value::Nil))
                        }
                        Value::Class(Class { ident, methods }) => {
                            let funcs = methods.iter().filter_map(|stmt| match stmt {
                                Stmt::FunDecl { ident, parameters, body } => {
                                    Some((*ident, Value::Func(Func { 
                                        ident,
                                        body: body.clone(),
                                        parameters: parameters.clone(),
                                        closure: self.env.clone(),
                                    })))
                                }
                                _ => None,
                            }).collect::<Vec<(&'src str, Value<'src>)>>();

                            let instance = Value::ClassInstance(Rc::new(RefCell::new(ClassInstance {
                                class_ident: ident,
                                fields: HashMap::from_iter(funcs),
                            })));

                            Ok(instance)
                        }
                        _ => Err(RuntimeError::UncallableType),
                    }
                    _ => Err(RuntimeError::TypeMismatch("Unsupported operands for this operation".to_owned())),
                }
            },
        }
    }

    pub fn exec_stmt(&mut self, stmt: Stmt<'src>) -> Result<Option<Value<'src>>, RuntimeError> {
        match stmt {
            Stmt::Print(expr) => {
                let val = self.eval_expr(&expr)?;
                let _ = writeln!(self.out, "{}", val);
                Ok(None)
            }
            Stmt::Expr(e) => {
                self.eval_expr(&e)?;
                Ok(None)
            }
            Stmt::Var { name, init_val } => {
                let val = self.eval_expr(&init_val)?;
                self.env.borrow_mut().set(name, val);
                Ok(None)
            }
            Stmt::Block(statements) => {
                let prev_env = self.env.clone();
                self.env = Rc::new(RefCell::new(Env::new_with_outer(self.env.clone())));
                let mut value = None;
                for stmt in statements {
                    if let Some(v) = self.exec_stmt(stmt)? {
                        value = Some(v);
                    }
                }
                self.env = prev_env;
                Ok(value)
            }
            Stmt::If {
                condition,
                stmt,
                else_stmt,
            } => {
                if self.eval_expr(&condition)?.is_truthy() {
                    self.exec_stmt(*stmt)
                } else if let Some(else_stmt) = else_stmt {
                    self.exec_stmt(*else_stmt)
                } else {
                    Ok(None)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    if self.eval_expr(&condition)?.is_truthy() {
                        match *body.clone() {
                            block @ Stmt::Block(_) => {
                                if let Some(val) = self.exec_stmt(block)? {
                                    return Ok(Some(val));
                                }
                            }
                            stmt => {
                                if let Some(val) = self.exec_stmt(stmt.clone())? {
                                    return Ok(Some(val));
                                }
                            }
                        };
                    } else {
                        break;
                    }
                }
                Ok(None)
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                let prev_env = self.env.clone();
                self.env = Rc::new(RefCell::new(Env::new_with_outer(self.env.clone())));
                if let Some(init) = initializer {
                    self.exec_stmt(*init)?;
                }
                loop {
                    if self.eval_expr(&condition)?.is_truthy() {
                        match *body.clone() {
                            block @ Stmt::Block(_) => self.exec_stmt(block)?,
                            stmt => self.exec_stmt(stmt.clone())?,
                        };
                        if let Some(incr_expr) = &increment {
                            self.eval_expr(incr_expr)?;
                        }
                    } else {
                        break;
                    }
                }
                self.env = prev_env;
                Ok(None)
            }
            Stmt::FunDecl {
                ident,
                parameters,
                body,
            } => {
                let func = Value::Func(Func {
                    ident,
                    parameters,
                    body,
                    // Capture current environment
                    closure: self.env.clone(),
                });
                self.env.borrow_mut().set(ident, func);
                Ok(None)
            }
            Stmt::Return(expr) => match expr {
                Some(expr) => self.eval_expr(&expr).map(Option::from),
                None => Ok(Some(Value::Nil)),
            },
            Stmt::ClassDecl { ident, methods } => {
                let class = Value::Class(Class { ident, methods });
                self.env.borrow_mut().set(ident, class);
                Ok(None)
            }
        }
    }

    fn look_up_ident(&self, atom: &Atom) -> Option<Value<'src>> {
        if let Atom::Ident { name, .. } = atom {
            // If there is a local resolved by a resolver with a given distance - use that.
            // Otherwise we'll look up the entire linked list at runtime bottom to top.
            return match self.resolver.locals.get(&atom.ident_hash()) {
                Some(distance) => self.env.borrow().get_at(name, *distance),
                None => self.env.borrow().get(name),
            };
        }
        panic!("Can't look up atom of type other than Ident");
    }

    fn update_ident(&self, atom: &Atom<'src>, val: Value<'src>) -> Result<(), RuntimeError> {
        if let Atom::Ident { name, .. } = atom {
            let distance = self.resolver.locals.get(&atom.ident_hash()).unwrap();
            self.env.borrow_mut().update_at(name, val, *distance)?;
            return Ok(());
        }
        panic!("Can't look up atom of type other than Ident");
    }

    pub fn current_line(&self) -> usize {
        std::cmp::max(self.parser.current_line() - 1, self.resolver.current_line() - 1)
    }
}

impl Default for Runtime<'_, std::io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'src> {
    Number(f64),
    String(Cow<'src, str>),
    Bool(bool),
    Nil,
    Func(Func<'src>),
    NativeFunc(NativeFunc),
    Class(Class<'src>),
    ClassInstance(Rc<RefCell<ClassInstance<'src>>>),
}

impl<'src> Value<'src> {
    fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(bool) => *bool,
            Self::Nil => false,
            _ => true,
        }
    }
}

impl<'src> std::fmt::Display for Value<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
            // Self::Func(func) => write!(f, "{}({})", func.ident, func.parameters.join(", ")),
            Self::Func(func) => write!(f, "<fn {}>", func.ident),
            Self::NativeFunc(func) => write!(f, "<fn {}>", func),
            Self::Class(cls) => write!(f, "{}", cls.ident),
            Self::ClassInstance(instance) => write!(f, "{} instance", instance.borrow().class_ident),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func<'src> {
    pub ident: &'src str,
    pub parameters: Vec<&'src str>,
    pub body: Vec<Stmt<'src>>,
    pub closure: Rc<RefCell<Env<'src>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunc {
    Clock,
}

impl std::fmt::Display for NativeFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clock => write!(f, "clock"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class<'src> {
    pub ident: &'src str,
    pub methods: Vec<Stmt<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassInstance<'src> {
    pub class_ident: &'src str,
    fields: HashMap<&'src str, Value<'src>>,
    // methods: HashMap<&'src str, Func<'src>>,
}

impl<'src> ClassInstance<'src> {
    fn get(&self, property: &'src str) -> Result<Value<'src>, RuntimeError> {
        self.fields
            .get(property)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedProperty(property.to_owned()))
    }

    fn set(&mut self, property: &'src str, value: Value<'src>) {
        self.fields.insert(property, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn add() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Number(5.0));
    }

    #[test]
    fn bool() {
        let mut parser = Parser::new("true");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn nil() {
        let mut parser = Parser::new("nil");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Nil);
    }

    #[test]
    fn string() {
        let mut parser = Parser::new("\"hello world\"");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::String(Cow::Borrowed("hello world")));
    }

    #[test]
    fn number() {
        let mut parser = Parser::new("85");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Number(85.0));
    }

    #[test]
    fn group() {
        let mut parser = Parser::new("(\"hello world\")");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::String(Cow::Borrowed("hello world")));
    }

    mod minus {
        use super::*;

        #[test]
        fn minus_negation_number_produces_negative_number() {
            let mut parser = Parser::new("-73");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Number(-73.0));
        }

        #[test]
        fn minus_negation_string_returns_error() {
            let mut parser = Parser::new("-\"hello world\"");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr);
            assert_eq!(
                value,
                Err(RuntimeError::TypeMismatch(
                    "Operand must be a number".to_owned()
                ))
            );
        }

        #[test]
        fn minus_negation_bool_returns_error() {
            let mut parser = Parser::new("-true");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr);
            assert_eq!(
                value,
                Err(RuntimeError::TypeMismatch(
                    "Operand must be a number".to_owned()
                ))
            );
        }
    }

    #[test]
    fn sub() {
        let mut parser = Parser::new("84 - 73");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Number(11.0));
    }

    #[test]
    fn bool_negation_produces_false() {
        let mut parser = Parser::new("!true");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(false));
    }

    #[test]
    fn non_zero_number_bang_negation_produces_false() {
        let mut parser = Parser::new("!73");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(false));
    }

    #[test]
    fn zero_number_bang_negation_produces_true() {
        let mut parser = Parser::new("!0");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn grouped_bool_negation_produces_false() {
        let mut parser = Parser::new("!((false))");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn nil_bang_negation_produces_true() {
        let mut parser = Parser::new("!nil");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn division() {
        let mut parser = Parser::new("42 / 5");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Number(8.4));
    }

    #[test]
    fn multiplication() {
        let mut parser = Parser::new("10 * 3");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Number(30.0));
    }

    #[test]
    fn mixed_mult_div() {
        let mut parser = Parser::new("(18 * 3 / (3 * 6))");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Number(3.0));
    }

    #[test]
    fn string_concat() {
        let mut parser = Parser::new("\"hello\" + \" world\"");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::String(Cow::Borrowed("hello world")));
    }

    #[test]
    fn less_produces_true_for_lower_left_value() {
        let mut parser = Parser::new("7 < 10");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn less_eq_produces_true_for_same_values() {
        let mut parser = Parser::new("7 <= 7");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn greater_produces_true_for_greater_left_value() {
        let mut parser = Parser::new("11 > 5");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn greater_eq_produces_true_for_same_values() {
        let mut parser = Parser::new("7 >= 7");
        let expr = parser.next_expr().unwrap();
        let value = Runtime::default().eval_expr(&expr).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    mod equality {
        use super::*;

        #[test]
        fn produces_true_for_same_numbers() {
            let mut parser = Parser::new("7 == 7");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_different_numbers() {
            let mut parser = Parser::new("7 == 8");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_same_strings() {
            let mut parser = Parser::new("\"hello\" == \"hello\"");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_different_strings() {
            let mut parser = Parser::new("\"hello\" == \"world\"");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_false_for_different_types() {
            let mut parser = Parser::new("\"hello\" == 1");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_when_both_are_true() {
            let mut parser = Parser::new("true == true");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_different_bools() {
            let mut parser = Parser::new("true == false");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }
    }

    mod inequality {
        use super::*;

        #[test]
        fn produces_false_for_same_numbers() {
            let mut parser = Parser::new("7 != 7");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_different_numbers() {
            let mut parser = Parser::new("7 != 8");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_same_strings() {
            let mut parser = Parser::new("\"hello\" != \"hello\"");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_different_strings() {
            let mut parser = Parser::new("\"hello\" != \"world\"");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_true_for_different_types() {
            let mut parser = Parser::new("\"hello\" != 1");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_when_both_are_true() {
            let mut parser = Parser::new("true != true");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_different_bools() {
            let mut parser = Parser::new("true != false");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }
    }

    mod and {
        use super::*;

        #[test]
        fn true_and_false_produce_false() {
            let mut parser = Parser::new("true and false");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn true_and_true_produce_true() {
            let mut parser = Parser::new("true and true");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn short_circuits_and_returns_first_untruthy_value() {
            let mut parser = Parser::new("false and 96");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn short_circuits_and_returns_first_untruthy_value_sequenced() {
            let mut parser = Parser::new("false and 96 and \"\"");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn returns_last_value_if_all_values_are_truthy() {
            let mut parser = Parser::new("10 and 20 and \"abc\" and 12");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Number(12.0));
        }
    }

    mod or {
        use super::*;

        #[test]
        fn true_or_false_produce_true() {
            let mut parser = Parser::new("true or false");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn false_or_false_produce_false() {
            let mut parser = Parser::new("false or false");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn short_circuits_and_returns_first_truthy_value() {
            let mut parser = Parser::new("96 or true");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Number(96.0));
        }

        #[test]
        fn short_circuits_and_returns_first_truthy_value_sequenced() {
            let mut parser = Parser::new("96 or true or 0 or 12");
            let expr = parser.next_expr().unwrap();
            let value = Runtime::default().eval_expr(&expr).unwrap();
            assert_eq!(value, Value::Number(96.0));
        }
    }

    mod statements {
        use super::*;

        #[test]
        fn print_statements_writes_to_out() {
            let cases = [
                ("print true;", "true\n"),
                ("print \"world\" + \"baz\" + \"bar\";", "worldbazbar\n"),
                ("print 27 - 26;", "1\n"),
                ("print \"bar\" == \"quz\";", "false\n"),
                ("print \"hello\"; print true;", "hello\ntrue\n"),
            ];

            for (src, expected) in cases {
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, expected, "unexpected output for: {src}");
            }
        }

        #[test]
        fn print_statement_can_print_multiline_string() {
            let mut buf = Vec::new();
            let mut runtime = Runtime::with_out(&mut buf);
            runtime.run("print \"a\nb\nc\n\";").unwrap();
            let actual = std::str::from_utf8(&buf).unwrap();
            assert_eq!(actual, "a\nb\nc\n\n");
        }

        #[test]
        fn parses_var_declaration_and_can_print_it_to_out() {
            let src = "var a = \"foo\";
                       print a;";
            let mut buf = Vec::new();
            let mut runtime = Runtime::with_out(&mut buf);
            runtime.run(src).unwrap();
            let actual = std::str::from_utf8(&buf).unwrap();
            assert_eq!(actual, "foo\n");
        }

        #[test]
        fn assignment_expression_changes_var_value() {
            let src = "var x = 10; x = 20; print x;";
            let mut buf = Vec::new();
            let mut runtime = Runtime::with_out(&mut buf);
            runtime.run(src).unwrap();
            let actual = std::str::from_utf8(&buf).unwrap();
            assert_eq!(actual, "20\n");
        }

        #[test]
        fn sequential_assignments_change_all_vars_accordingly() {
            let src = "
                var a = 0; 
                var b = 1; 
                a = b = 10;
                print a;
                print b;
            ";
            let mut buf = Vec::new();
            let mut runtime = Runtime::with_out(&mut buf);
            runtime.run(src).unwrap();
            let actual = std::str::from_utf8(&buf).unwrap();
            assert_eq!(actual, "10\n10\n");
        }
    }

    mod environments {
        use super::*;

        #[test]
        fn local_vars_overshadow_ones_from_outer_environemnt() {
            let src = "
                {
                    var world = \"before\";
                    {
                        var world = \"after\";
                        print world;
                    }
                    print world;
                }
            ";
            let mut buf = Vec::new();
            let mut runtime = Runtime::with_out(&mut buf);
            runtime.run(src).unwrap();
            let actual = std::str::from_utf8(&buf).unwrap();
            assert_eq!(actual, "after\nbefore\n");
        }
    }

    mod control_flow {
        use super::*;

        mod if_else {
            use super::*;

            #[test]
            fn executes_stmt_if_condition_is_true() {
                let src = "
                if (true) {
                    print \"good\";
                } else {
                    print \"bad\";
                }
            ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "good\n");
            }

            #[test]
            fn executes_else_stmt_if_condition_is_false() {
                let src = "
                if (false) {
                    print \"good\";
                } else {
                    print \"bad\";
                }
            ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "bad\n");
            }
        }

        mod while_stmt {
            use super::*;

            #[test]
            fn loops_over_body_while_a_given_condition_is_true() {
                let src = "
                    var a = 0;
                    while (a < 3) {
                        print \"hello\";
                        a = a + 1;
                    }
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "hello\nhello\nhello\n");
            }

            #[test]
            fn loops_over_statement_while_a_given_condition_is_true() {
                let src = "
                    var a = 0;
                    while (a < 3) print a = a + 1;
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "1\n2\n3\n");
            }
        }

        mod for_stmt {
            use super::*;

            #[test]
            fn loops_over_body_while_a_given_condition_is_true() {
                let src = "
                    for (var i = 0; i < 3; i = i + 1) {
                        print i;
                    }
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "0\n1\n2\n");
            }
        }

        mod function_call {
            use super::*;

            #[test]
            fn built_in_clock_fn_returns_timestamp() {
                let src = "print clock();";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert!(!actual.is_empty());
            }

            #[test]
            fn calls_declared_function() {
                let src = "
                    fun test_func() {
                        print \"hello from test_func\";
                    }
                    test_func();
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "hello from test_func\n");
            }

            #[test]
            fn func_invocation_produces_a_value_returned_from_func_body() {
                let src = "
                    fun foo() { return 10; }
                    print foo();    
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "10\n");
            }

            #[test]
            fn func_does_not_execute_statements_after_returning() {
                let src = "
                    fun foo() {
                        if (true) {
                            return \"from if\";
                        }
                        return \"outside of if\";
                    }
                    print foo();    
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "from if\n");
            }

            #[test]
            fn handles_recursive_calls_with_return_statements() {
                let src = "
                    fun fib(n) {
                        if (n < 2) return n;
                        return fib(n - 2) + fib(n - 1);
                    }

                    var start = clock();
                    print fib(10) == 55;
                    print (clock() - start) < 5; // 5 seconds
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "true\ntrue\n");
            }

            #[test]
            fn returns_from_while_block() {
                let src = "
                    fun f() {
                        while (!false) return \"ok\";
                    }
                    print f();
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "ok\n");
            }

            #[test]
            fn returns_nil_from_func_if_skipped_expr_after_return_keyword() {
                let src = "
                    fun f() {
                        return;
                        print \"bad\";
                    }
                 
                    print f();
                ";
                let mut buf = Vec::new();
                let mut runtime = Runtime::with_out(&mut buf);
                runtime.run(src).unwrap();
                let actual = std::str::from_utf8(&buf).unwrap();
                assert_eq!(actual, "nil\n");
            }
        }
    }
}
