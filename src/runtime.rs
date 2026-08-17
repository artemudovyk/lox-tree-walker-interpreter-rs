use crate::parser::{Atom, Expr, Op};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'src> {
    Number(f64),
    String(&'src str),
    Bool(bool),
    Nil,
}

impl<'src> From<&Atom<'src>> for Value<'src> {
    fn from(value: &Atom<'src>) -> Self {
        match value {
            Atom::Number(n) => Value::Number(*n),
            Atom::String(s) => Value::String(s),
            Atom::Bool(b) => Value::Bool(*b),
            Atom::Nil => Value::Nil,
        }
    }
}

impl<'src> std::fmt::Display for Value<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => {
                if *n == n.trunc() {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
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
                Op::Add => {
                    let mut acc = 0.0;
                    for arg in args.iter().map(|a| a.evaluate()) {
                        match arg? {
                            Value::Number(n) => acc += n,
                            _ => panic!("unexpected value"),
                        }
                    }
                    Ok(Value::Number(acc))
                }
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
        assert_eq!(value, Value::String("hello world"));
    }
}
