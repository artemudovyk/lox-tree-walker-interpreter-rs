use crate::parser::{Atom, Expr, Op, ParseError, Parser, Stmt};
use std::{borrow::Cow, collections::HashMap, io::Write};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum RuntimeError {
    #[error("{0}")]
    TypeMismatch(String),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("variable {0} is undeclared")]
    UndeclaredVar(String),
}

pub struct Env<'src> {
    map: HashMap<&'src str, Value<'src>>,
}

impl<'src> Env<'src> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value<'src>> {
        self.map.get(key)
    }

    pub fn set(&mut self, key: &'src str, val: Value<'src>) {
        self.map.insert(key, val);
    }

    pub fn update(&mut self, key: &'src str, val: Value<'src>) -> Result<(), RuntimeError> {
        if !self.map.contains_key(key) {
            return Err(RuntimeError::UndeclaredVar(key.to_owned()));
        }
        self.set(key, val);
        Ok(())
    }
}

pub struct Runtime<'src, W: Write> {
    out: W,
    env: Env<'src>,
}

impl<'src, W: Write> Runtime<'src, W> {
    pub fn with_out(writer: W) -> Self {
        Self {
            out: writer,
            env: Env::new(),
        }
    }

    pub fn run(&mut self, src: &'src str) -> Result<(), RuntimeError> {
        let mut parser = Parser::new(src);
        while let Some(stmt) = parser.next_stmt()? {
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
                Atom::Ident(ident) => Ok(self
                    .env
                    .get(ident)
                    .map(|v| v.to_owned())
                    .ok_or(RuntimeError::UndeclaredVar(ident.to_string()))?),
            },
            Expr::Cons(op, args) => match op {
                Op::Add => match args.as_slice() {
                    [left, right] => match (self.eval_expr(left)?, self.eval_expr(right)?) {
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
                        (Expr::Atom(Atom::Ident(ident)), val) => {
                            self.env.update(ident, val.clone())?; Ok(val)
                        }
                        _ => Err(RuntimeError::TypeMismatch("Assignment must be done on the variable and have an expression on the right side".to_owned()))
                    },
                    _ => Err(RuntimeError::TypeMismatch("Assignment must be done on the variable and have an expression on the right side".to_owned()))
                }
            },
        }
    }

    pub fn exec_stmt(&mut self, stmt: Stmt<'src>) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Print(expr) => {
                let val = self.eval_expr(&expr)?;
                let _ = writeln!(self.out, "{}", val);
            }
            Stmt::Expr(e) => {
                self.eval_expr(&e)?;
            }
            Stmt::Var { name, init_val } => {
                let val = self.eval_expr(&init_val)?;
                self.env.set(name, val);
            }
        }
        Ok(())
    }
}

impl Default for Runtime<'_, std::io::Stdout> {
    fn default() -> Self {
        Self {
            out: std::io::stdout(),
            env: Env::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'src> {
    Number(f64),
    String(Cow<'src, str>),
    Bool(bool),
    Nil,
}

impl<'src> std::fmt::Display for Value<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl<'src> Stmt<'src> {}

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
}
