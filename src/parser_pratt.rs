use crate::lexer::{Keyword, Lexer, Literal, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Atom<'src> {
    String(&'src str),
    Number(f64),
    Bool(bool),
    Nil,
}

impl<'src> std::fmt::Display for Atom<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) if *n == n.trunc() => write!(f, "{:.01}", n),
            Self::Number(n) => write!(f, "{}", n),
            Self::Bool(bool) => write!(f, "{}", bool),
            Self::Nil => write!(f, "nil"),
        }
    }
}

/// Operation
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Mult,
    Group,
    Bang,
    Minus,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Mult => write!(f, "*"),
            Self::Group => write!(f, "group"),
            Self::Bang => write!(f, "!"),
            Self::Minus => write!(f, "-"),
        }
    }
}

/// S-expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
    Atom(Atom<'src>),
    /// Construct
    Cons(Op, Vec<Expr<'src>>),
}

impl<'src> std::fmt::Display for Expr<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Atom(a) => write!(f, "{}", a),
            Expr::Cons(head, rest) => {
                write!(f, "({}", head)?;
                for s in rest {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}

#[allow(dead_code)]
pub struct Parser<'src> {
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

    pub fn parse(&mut self) -> Expr<'src> {
        self.exprs_bp(0)
    }

    pub fn exprs_bp(&mut self, min_bp: u8) -> Expr<'src> {
        let mut lhs = loop {
            match self.lexer.next_token().kind {
                TokenKind::Literal(Literal::Number(n)) => break Expr::Atom(Atom::Number(n)),
                TokenKind::Literal(Literal::String { s, .. }) => break Expr::Atom(Atom::String(s)),
                TokenKind::LeftParen => {
                    let lhs = self.exprs_bp(0);
                    self.lexer.advance();
                    break Expr::Cons(Op::Group, vec![lhs]);
                }
                TokenKind::Bang => {
                    let op = Op::Bang;
                    let (_, r_bp) = prefix_binding_power(&op);
                    let rhs = self.exprs_bp(r_bp);
                    self.lexer.advance();
                    break Expr::Cons(op, vec![rhs]);
                }
                TokenKind::Minus => {
                    let op = Op::Minus;
                    let (_, r_bp) = prefix_binding_power(&op);
                    let rhs = self.exprs_bp(r_bp);
                    self.lexer.advance();
                    break Expr::Cons(op, vec![rhs]);
                }
                TokenKind::Keyword(Keyword::True) => return Expr::Atom(Atom::Bool(true)),
                TokenKind::Keyword(Keyword::False) => return Expr::Atom(Atom::Bool(false)),
                TokenKind::Keyword(Keyword::Nil) => return Expr::Atom(Atom::Nil),
                TokenKind::Whitespace => continue,
                t => panic!("bad lhs token: {:?}", t),
            }
        };

        loop {
            let op = match &self.lexer.peek().kind {
                TokenKind::Plus => Op::Add,
                TokenKind::Star => Op::Mult,
                TokenKind::RightParen => break,
                TokenKind::Whitespace => {
                    self.lexer.advance();
                    continue;
                }
                TokenKind::Eof => break,
                t => panic!("bad op token: {:?}", t),
            };

            if let Some((l_bp, r_bp)) = infix_binding_power(&op) {
                // If true, we finished with this subtree
                if l_bp < min_bp {
                    break;
                }
                // Otherwise advance token cursor and make a recursive call to build rhs
                self.lexer.advance();
                let rhs = self.exprs_bp(r_bp);
                lhs = Expr::Cons(op, vec![lhs, rhs]);
                continue;
            }

            break;
        }

        lhs
    }

    pub fn next_expr(&mut self) -> Expr<'src> {
        self.exprs_bp(0)
    }
}

fn infix_binding_power(op: &Op) -> Option<(u8, u8)> {
    match op {
        Op::Add => Some((1, 2)),
        Op::Mult => Some((3, 4)),
        _ => None,
    }
}

fn prefix_binding_power(op: &Op) -> ((), u8) {
    match op {
        Op::Bang => ((), 11),
        _ => ((), 0),
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
    fn group() {
        let mut parser = Parser::new("(\"foo\")");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(group foo)");
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
}
