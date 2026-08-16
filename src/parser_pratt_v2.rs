#![allow(dead_code)]
use crate::lexer::{Keyword, Lexer, Literal, Token, TokenKind};

#[derive(Debug)]
enum Expr<'src> {
    Atom(Atom<'src>),
    Cons(Op, Vec<Expr<'src>>),
}

impl<'src> std::fmt::Display for Expr<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atom(a) => write!(f, "{}", a),
            Self::Cons(op, atoms) => {
                write!(f, "({}", op)?;
                for atom in atoms {
                    write!(f, " {}", atom)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug)]
enum Atom<'src> {
    Number(f64),
    String(&'src str),
    Bool(bool),
    Nil,
}

impl<'src> std::fmt::Display for Atom<'src> {
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

impl<'src> TryFrom<&Token<'src>> for Atom<'src> {
    type Error = ();

    fn try_from(value: &Token<'src>) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Literal(Literal::Number(n)) => Ok(Atom::Number(n)),
            TokenKind::Literal(Literal::String { s, .. }) => Ok(Atom::String(s)),
            TokenKind::Keyword(Keyword::True) => Ok(Atom::Bool(true)),
            TokenKind::Keyword(Keyword::False) => Ok(Atom::Bool(false)),
            TokenKind::Keyword(Keyword::Nil) => Ok(Atom::Nil),
            _ => Err(()),
        }
    }
}

impl<'src> TryFrom<Token<'src>> for Atom<'src> {
    type Error = ();

    fn try_from(value: Token<'src>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

#[derive(Debug)]
enum Op {
    Add,
    Mult,
    Div,
    Group,
    Bang,
    Minus,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mult => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Group => write!(f, "group"),
            Self::Bang => write!(f, "!"),
        }
    }
}

impl Op {
    fn infix_bp(&self) -> Option<(u8, u8)> {
        match self {
            Self::Add | Self::Minus => Some((1, 2)),
            Self::Mult | Self::Div => Some((3, 4)),
            _ => None,
        }
    }

    fn prefix_bp(&self) -> Option<u8> {
        match self {
            Self::Bang => Some(10),
            Self::Minus => Some(10),
            _ => None,
        }
    }
}

impl<'src> TryFrom<&Token<'src>> for Op {
    type Error = ();

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Plus => Ok(Op::Add),
            TokenKind::Star => Ok(Op::Mult),
            TokenKind::Bang => Ok(Op::Bang),
            TokenKind::Minus => Ok(Op::Minus),
            TokenKind::Slash => Ok(Op::Div),
            _ => Err(()),
        }
    }
}

impl<'src> TryFrom<Token<'src>> for Op {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

struct Parser<'src> {
    src: &'src str,
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            lexer: Lexer::new(src),
        }
    }

    pub fn next_expr(&mut self) -> Expr<'src> {
        self.expr_bp(0)
    }

    pub fn expr_bp(&mut self, min_bp: u8) -> Expr<'src> {
        let mut lhs = {
            let token = self.lexer.next_token();
            if let Ok(atom) = Atom::try_from(&token) {
                Expr::Atom(atom)
            } else if matches!(token.kind, TokenKind::LeftParen) {
                let members = self.expr_bp(0);
                assert!(
                    matches!(self.lexer.next_token().kind, TokenKind::RightParen),
                    "unterminated group"
                );
                Expr::Cons(Op::Group, vec![members])
            } else if let Ok(op) = Op::try_from(&token)
                && let Some(bp) = op.prefix_bp()
            {
                let rhs = self.expr_bp(bp);
                Expr::Cons(op, vec![rhs])
            } else {
                panic!("no lhs")
            }
        };

        loop {
            let peeked = self.lexer.peek();
            if let Ok(op) = Op::try_from(peeked)
                && let Some((l_bp, r_bp)) = op.infix_bp()
            {
                // If the next operator's left binding power is less than minimum, it is not strong
                // enough to bind the next atom, so we break and return lhs we've built so far
                if l_bp < min_bp {
                    eprintln!("lbp {l_bp} < min_bp {min_bp}");
                    break;
                }
                self.lexer.advance();
                let rhs = self.expr_bp(r_bp);
                // What is built so far
                lhs = Expr::Cons(op, vec![lhs, rhs]);
                // Continue to check the next operator
                continue;
            } else if matches!(peeked.kind, TokenKind::RightParen) || peeked.is_eof() {
                break;
            }

            break;
        }

        // Return what was built so far as lhs: if nothing was built in a loop above, then it will
        // be an atom found at the beginning of the function. Otherwise it will be constructed
        // expression for this subtree bound by binding power.
        lhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(+ 2.0 3.0)");
    }

    #[test]
    fn add_mult() {
        let mut parser = Parser::new("2 + 3 * 4");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(+ 2.0 (* 3.0 4.0))");
    }

    #[test]
    fn add_mult_add() {
        let mut parser = Parser::new("2 + 3 * 4 + 5");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(+ (+ 2.0 (* 3.0 4.0)) 5.0)");
    }

    #[test]
    fn k_true() {
        let mut parser = Parser::new("true");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "true");
    }

    #[test]
    fn k_false() {
        let mut parser = Parser::new("false");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "false");
    }

    #[test]
    fn nil() {
        let mut parser = Parser::new("nil");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "nil");
    }

    #[test]
    fn number() {
        let mut parser = Parser::new("42.47");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "42.47");

        let mut parser = Parser::new("42");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "42.0");
    }

    #[test]
    fn zero() {
        let mut parser = Parser::new("0.0");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "0.0");
    }

    #[test]
    fn string() {
        let mut parser = Parser::new("\"hello\"");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "hello");
    }

    #[test]
    fn group_single_member() {
        let mut parser = Parser::new("(15)");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(group 15.0)");
    }

    #[test]
    fn group_multiple_members() {
        let mut parser = Parser::new("(10.1 + 21.2)");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(group (+ 10.1 21.2))");
    }

    #[test]
    fn bang_bool() {
        let mut parser = Parser::new("!true");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(! true)");
    }

    #[test]
    fn minus_number() {
        let mut parser = Parser::new("-70");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(- 70.0)");
    }

    #[test]
    fn mult_div() {
        let mut parser = Parser::new("16 * 38 / 58");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(/ (* 16.0 38.0) 58.0)");
    }

    #[test]
    fn mix() {
        let mut parser = Parser::new("(82 * -27 / (34 * 66))");
        let expr = parser.next_expr();
        assert_eq!(
            format!("{expr}"),
            "(group (/ (* 82.0 (- 27.0)) (group (* 34.0 66.0))))"
        );
    }
}
