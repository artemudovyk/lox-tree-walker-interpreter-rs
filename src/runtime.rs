use std::borrow::Cow;

use crate::parser::{Atom, Expr, Op};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("type mismatch")]
    TypeMismatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'src> {
    Number(f64),
    String(Cow<'src, str>),
    Bool(bool),
    Nil,
}

impl<'src> From<&Atom<'src>> for Value<'src> {
    fn from(value: &Atom<'src>) -> Self {
        match value {
            Atom::Number(n) => Value::Number(*n),
            Atom::String(s) => Value::String(Cow::Borrowed(s)),
            Atom::Bool(b) => Value::Bool(*b),
            Atom::Nil => Value::Nil,
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
        }
    }
}

impl<'src> Expr<'src> {
    pub fn evaluate(&self) -> Result<Value<'src>, RuntimeError> {
        match self {
            Self::Atom(atom) => Ok(Value::from(atom)),
            Self::Cons(op, args) => match op {
                Op::Add => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => {
                            Ok(Value::String(Cow::Owned(format!("{l}{r}"))))
                        }
                        _ => Err(RuntimeError::TypeMismatch)?,
                    },
                    _ => Err(RuntimeError::TypeMismatch)?,
                },
                Op::Group => match args.as_slice() {
                    [arg] => arg.evaluate(),
                    _ => Err(RuntimeError::TypeMismatch)?,
                },
                Op::Minus => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    [arg] => match arg.evaluate()? {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    _ => Err(RuntimeError::TypeMismatch),
                },
                Op::Div => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    _ => Err(RuntimeError::TypeMismatch),
                },
                Op::Mult => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    _ => Err(RuntimeError::TypeMismatch),
                },
                Op::Bang => match args.as_slice() {
                    [arg] => match arg.evaluate()? {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        Value::Number(n) => Ok(Value::Bool(n == 0.0)),
                        Value::Nil => Ok(Value::Bool(true)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    _ => Err(RuntimeError::TypeMismatch)?,
                },
                _ => panic!("unimplemented operation"),
            },
        }
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
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(5.0));
    }

    #[test]
    fn bool() {
        let mut parser = Parser::new("true");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn nil() {
        let mut parser = Parser::new("nil");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Nil);
    }

    #[test]
    fn string() {
        let mut parser = Parser::new("\"hello world\"");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::String(Cow::Borrowed("hello world")));
    }

    #[test]
    fn number() {
        let mut parser = Parser::new("85");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(85.0));
    }

    #[test]
    fn group() {
        let mut parser = Parser::new("(\"hello world\")");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::String(Cow::Borrowed("hello world")));
    }

    #[test]
    fn number_minus_negation_produces_negative_number() {
        let mut parser = Parser::new("-73");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(-73.0));
    }

    #[test]
    fn sub() {
        let mut parser = Parser::new("84 - 73");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(11.0));
    }

    #[test]
    fn bool_negation_produces_false() {
        let mut parser = Parser::new("!true");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(false));
    }

    #[test]
    fn non_zero_number_bang_negation_produces_false() {
        let mut parser = Parser::new("!73");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(false));
    }

    #[test]
    fn zero_number_bang_negation_produces_true() {
        let mut parser = Parser::new("!0");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn grouped_bool_negation_produces_false() {
        let mut parser = Parser::new("!((false))");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn nil_bang_negation_produces_true() {
        let mut parser = Parser::new("!nil");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn division() {
        let mut parser = Parser::new("42 / 5");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(8.4));
    }

    #[test]
    fn multiplication() {
        let mut parser = Parser::new("10 * 3");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(30.0));
    }

    #[test]
    fn mixed_mult_div() {
        let mut parser = Parser::new("(18 * 3 / (3 * 6))");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Number(3.0));
    }

    #[test]
    fn string_concat() {
        let mut parser = Parser::new("\"hello\" + \" world\"");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::String(Cow::Borrowed("hello world")));
    }
}
