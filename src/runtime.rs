use std::borrow::Cow;

use crate::parser::{Atom, Expr, Op};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum RuntimeError {
    #[error("{0}")]
    TypeMismatch(String),
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
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Group => match args.as_slice() {
                    [arg] => arg.evaluate(),
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Minus => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    [arg] => match arg.evaluate()? {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Operand must be a number".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Div => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Mult => match args.as_slice() {
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => Err(RuntimeError::TypeMismatch(
                            "Unsupported operand types for this operation.".to_owned(),
                        ))?,
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
                Op::Bang => match args.as_slice() {
                    [arg] => match arg.evaluate()? {
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
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
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
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
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
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
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
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
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
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
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
                    [left, right] => match (left.evaluate()?, right.evaluate()?) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l != r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::Bool(l != r)),
                        (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l != r)),
                        _ => Ok(Value::Bool(true)),
                    },
                    _ => Err(RuntimeError::TypeMismatch(
                        "Unsupported operand types for this operation.".to_owned(),
                    ))?,
                },
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

    mod minus {
        use super::*;

        #[test]
        fn minus_negation_number_produces_negative_number() {
            let mut parser = Parser::new("-73");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Number(-73.0));
        }

        #[test]
        fn minus_negation_string_returns_error() {
            let mut parser = Parser::new("-\"hello world\"");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate();
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
            let value = expr.evaluate();
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

    #[test]
    fn less_produces_true_for_lower_left_value() {
        let mut parser = Parser::new("7 < 10");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn less_eq_produces_true_for_same_values() {
        let mut parser = Parser::new("7 <= 7");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn greater_produces_true_for_greater_left_value() {
        let mut parser = Parser::new("11 > 5");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn greater_eq_produces_true_for_same_values() {
        let mut parser = Parser::new("7 >= 7");
        let expr = parser.next_expr().unwrap();
        let value = expr.evaluate().unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    mod equality {
        use super::*;

        #[test]
        fn produces_true_for_same_numbers() {
            let mut parser = Parser::new("7 == 7");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_different_numbers() {
            let mut parser = Parser::new("7 == 8");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_same_strings() {
            let mut parser = Parser::new("\"hello\" == \"hello\"");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_different_strings() {
            let mut parser = Parser::new("\"hello\" == \"world\"");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_false_for_different_types() {
            let mut parser = Parser::new("\"hello\" == 1");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_when_both_are_true() {
            let mut parser = Parser::new("true == true");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_different_bools() {
            let mut parser = Parser::new("true == false");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }
    }

    mod inequality {
        use super::*;

        #[test]
        fn produces_false_for_same_numbers() {
            let mut parser = Parser::new("7 != 7");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_different_numbers() {
            let mut parser = Parser::new("7 != 8");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_for_same_strings() {
            let mut parser = Parser::new("\"hello\" != \"hello\"");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_different_strings() {
            let mut parser = Parser::new("\"hello\" != \"world\"");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_true_for_different_types() {
            let mut parser = Parser::new("\"hello\" != 1");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }

        #[test]
        fn produces_false_when_both_are_true() {
            let mut parser = Parser::new("true != true");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(false));
        }

        #[test]
        fn produces_true_for_different_bools() {
            let mut parser = Parser::new("true != false");
            let expr = parser.next_expr().unwrap();
            let value = expr.evaluate().unwrap();
            assert_eq!(value, Value::Bool(true));
        }
    }
}
