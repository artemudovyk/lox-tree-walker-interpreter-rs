use crate::lexer::{Lexer, Literal, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Number(f64),
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
        }
    }
}

/// Operation
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Mult,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Mult => write!(f, "*"),
        }
    }
}

/// S-expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Atom(Atom),
    /// Construct
    Cons(Op, Vec<Expr>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Atom(i) => write!(f, "{}", i),
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

    pub fn parse(&mut self) -> Expr {
        self.exprs_bp(0)
    }

    pub fn exprs_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = loop {
            match dbg!(self.lexer.next_token()).kind {
                TokenKind::Literal(Literal::Number(n)) => break Expr::Atom(Atom::Number(n)),
                TokenKind::Whitespace => continue,
                t => panic!("bad lhs token: {:?}", t),
            }
        };

        loop {
            let op = match &self.lexer.peek().kind {
                TokenKind::Plus => Op::Add,
                TokenKind::Star => Op::Mult,
                TokenKind::Whitespace => {
                    self.lexer.advance();
                    continue;
                }
                TokenKind::Eof => break,
                t => panic!("bad op token: {:?}", t),
            };

            let (l_bp, r_bp) = infix_binding_power(&op);
            // If true, we finished with this subtree
            if l_bp < min_bp {
                break;
            }
            // Otherwise advance token cursor and make a recursive call to build rhs
            self.lexer.advance();
            let rhs = self.exprs_bp(r_bp);

            lhs = Expr::Cons(op, vec![lhs, rhs]);
        }

        lhs
    }

    pub fn next_expr(&mut self) -> Expr {
        self.exprs_bp(0)
    }
}

fn infix_binding_power(op: &Op) -> (u8, u8) {
    match op {
        Op::Add => (1, 2),
        Op::Mult => (3, 4),
        // _ => panic!("bad op: {:?}", op),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(+ 2 3)");
    }

    #[test]
    fn add_mult() {
        let mut parser = Parser::new("2 + 3 * 4");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(+ 2 (* 3 4))");
    }

    #[test]
    fn add_mult_add() {
        let mut parser = Parser::new("2 + 3 * 4 + 5");
        let expr = parser.next_expr();
        assert_eq!(format!("{expr}"), "(+ (+ 2 (* 3 4)) 5)");
    }
}
