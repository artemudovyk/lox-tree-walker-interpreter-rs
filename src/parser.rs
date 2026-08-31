#![allow(dead_code)]
use crate::lexer::{Keyword, Lexer, Literal, Token, TokenKind};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("[line {line}] Error at '{lexeme}': Expect expression.")]
    UnexpectedToken { line: usize, lexeme: String },
    #[error("[line {line}] Error at '{lexeme}'. Expect ')'.")]
    UnmatchedParen { line: usize, lexeme: String },
    #[error("[line {line}] Error at '{lexeme}'. Unterminated string.")]
    UnterminatedString { line: usize, lexeme: String },
    // TODO: somewhat collides with UnexpectedToken - think about a better name for both
    // I want for this error message
    #[error("{0}")]
    ExpectedTokenNotFound(String),
    #[error("[line {line}] Error at end: Expect '}}'.")]
    UnclosedBlock { line: usize },
    #[error("[line {line}] Expected statement after if condition.")]
    EmptyIfStatement { line: usize },
    #[error("[line {line}] Expected statement after while condition.")]
    EmptyWhileStatement { line: usize },
    #[error("Too many arguments. Functions can't have more than 255 arguments.")]
    TooManyArguments,
    #[error("Missing comma between parameters in function declaration.")]
    MissingCommaBetweenParameters,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'src> {
    Expr(Expr<'src>),
    Print(Expr<'src>),
    Var {
        name: &'src str,
        init_val: Expr<'src>,
    },
    Block(Vec<Stmt<'src>>),
    If {
        condition: Expr<'src>,
        stmt: Box<Stmt<'src>>,
        else_stmt: Option<Box<Stmt<'src>>>,
    },
    While {
        condition: Expr<'src>,
        body: Box<Stmt<'src>>,
    },
    For {
        initializer: Option<Box<Stmt<'src>>>,
        condition: Expr<'src>,
        increment: Option<Expr<'src>>,
        body: Box<Stmt<'src>>,
    },
    Return(Expr<'src>),
    FunDecl {
        ident: &'src str,
        parameters: Vec<&'src str>,
        body: Vec<Stmt<'src>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Atom<'src> {
    Number(f64),
    String(&'src str),
    Bool(bool),
    Nil,
    Ident(&'src str),
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
            Self::Ident(ident) => write!(f, "{}", ident),
        }
    }
}

impl<'src> TryFrom<&Token<'src>> for Atom<'src> {
    type Error = ();

    fn try_from(value: &Token<'src>) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Literal(Literal::Number(n)) => Ok(Atom::Number(n)),
            TokenKind::Literal(Literal::String { s, terminated }) => {
                if terminated {
                    Ok(Atom::String(s))
                } else {
                    Err(())
                }
            }
            TokenKind::Keyword(Keyword::True) => Ok(Atom::Bool(true)),
            TokenKind::Keyword(Keyword::False) => Ok(Atom::Bool(false)),
            TokenKind::Keyword(Keyword::Nil) => Ok(Atom::Nil),
            TokenKind::Ident => Ok(Atom::Ident(value.lexeme)),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Mult,
    Div,
    Group,
    Bang,
    Minus,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    EqEq,
    NotEq,
    Assign,
    And,
    Or,
    Call,
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
            Self::Less => write!(f, "<"),
            Self::LessEq => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEq => write!(f, ">="),
            Self::EqEq => write!(f, "=="),
            Self::NotEq => write!(f, "!="),
            Self::Assign => write!(f, "="),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Call => write!(f, "call"),
        }
    }
}

impl Op {
    fn infix_bp(&self) -> Option<(u8, u8)> {
        match self {
            Self::Assign => Some((2, 1)),
            Self::Or => Some((3, 2)),
            Self::And => Some((5, 4)),
            Self::Less
            | Self::LessEq
            | Self::Greater
            | Self::GreaterEq
            | Self::EqEq
            | Self::NotEq => Some((6, 7)),
            Self::Add | Self::Minus => Some((8, 9)),
            Self::Mult | Self::Div => Some((10, 11)),
            _ => None,
        }
    }

    fn prefix_bp(&self) -> Option<u8> {
        match self {
            Self::Bang => Some(20),
            Self::Minus => Some(20),
            _ => None,
        }
    }

    fn postfix_bp(&self) -> Option<u8> {
        match self {
            Self::Call => Some(20),
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
            TokenKind::Less => Ok(Op::Less),
            TokenKind::LessEqual => Ok(Op::LessEq),
            TokenKind::Greater => Ok(Op::Greater),
            TokenKind::GreaterEqual => Ok(Op::GreaterEq),
            TokenKind::EqEq => Ok(Op::EqEq),
            TokenKind::BangEq => Ok(Op::NotEq),
            TokenKind::Eq => Ok(Op::Assign),
            TokenKind::Keyword(Keyword::And) => Ok(Op::And),
            TokenKind::Keyword(Keyword::Or) => Ok(Op::Or),
            TokenKind::LeftParen => Ok(Op::Call),
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

    fn assert_semicolon(&mut self) {
        assert!(
            matches!(self.lexer.next_token().kind, TokenKind::Semicolon),
            "Expect ';' at the end of the statement."
        );
    }

    fn expect_token(
        &mut self,
        token_kind: TokenKind<'src>,
        msg: &str,
    ) -> Result<Token<'src>, ParseError> {
        let token = self.lexer.next_token();
        if token.kind != token_kind {
            return Err(ParseError::ExpectedTokenNotFound(format!(
                "[line {}] Error at '{}': {}",
                token.line, token.lexeme, msg
            )));
        }
        Ok(token)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt<'src>>, ParseError> {
        let mut statements = vec![];
        loop {
            match self.lexer.peek().kind {
                TokenKind::RightBrace => {
                    self.lexer.advance();
                    break;
                }
                // TODO: not sure about that, but I'll leave it here for a time being
                // need to think about a better way to check if block is properly closed
                TokenKind::Eof => {
                    return Err(ParseError::UnclosedBlock {
                        line: self.current_line(),
                    });
                }
                _ => statements.push(
                    self.next_stmt()?
                        .expect("Statement is expected at this point"),
                ),
            }
        }
        Ok(statements)
    }

    pub fn next_stmt(&mut self) -> Result<Option<Stmt<'src>>, ParseError> {
        loop {
            match self.lexer.peek().kind {
                TokenKind::Keyword(Keyword::Print) => {
                    self.lexer.advance(); // Consume `print` keyword
                    let expr = self.next_expr()?;
                    self.expect_token(TokenKind::Semicolon, "Expect expression.")?;
                    return Ok(Some(Stmt::Print(expr)));
                }
                TokenKind::Keyword(Keyword::Var) => {
                    self.lexer.advance(); // Consume `var` keyword
                    let ident = self.expect_token(TokenKind::Ident, "Expect identifier.")?;
                    let init_val = match self.lexer.peek().kind {
                        TokenKind::Eq => {
                            self.lexer.advance();
                            self.next_expr()?
                        }
                        TokenKind::Semicolon => Expr::Atom(Atom::Nil),
                        _ => Err(ParseError::UnexpectedToken {
                            line: self.current_line(),
                            lexeme: self.lexer.peek().lexeme.to_owned(),
                        })?,
                    };
                    self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                    return Ok(Some(Stmt::Var {
                        name: ident.lexeme,
                        init_val,
                    }));
                }
                TokenKind::Keyword(Keyword::If) => {
                    self.lexer.advance();
                    self.expect_token(TokenKind::LeftParen, "Expect '('.")?;
                    let cond = self.next_expr()?;
                    self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                    let stmt = match self.lexer.peek().kind {
                        TokenKind::LeftBrace => {
                            self.lexer.advance();
                            let statements = self.parse_block()?;
                            Box::new(Stmt::Block(statements))
                        }
                        _ => match self.next_stmt()? {
                            Some(s) => Box::new(s),
                            None => Err(ParseError::EmptyIfStatement {
                                line: self.current_line(),
                            })?,
                        },
                    };
                    let else_stmt = match self.lexer.peek().kind {
                        TokenKind::Keyword(Keyword::Else) => {
                            self.lexer.advance();
                            match self.lexer.peek().kind {
                                TokenKind::LeftBrace => {
                                    self.lexer.advance();
                                    let statements = self.parse_block()?;
                                    Some(Box::new(Stmt::Block(statements)))
                                }
                                _ => self.next_stmt()?.map(Box::new),
                            }
                        }
                        _ => None,
                    };
                    return Ok(Some(Stmt::If {
                        condition: cond,
                        stmt,
                        else_stmt,
                    }));
                }
                TokenKind::Keyword(Keyword::While) => {
                    self.lexer.advance();
                    self.expect_token(TokenKind::LeftParen, "Expect '('.")?;
                    let cond = self.next_expr()?;
                    self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                    let body = match self.lexer.peek().kind {
                        TokenKind::LeftBrace => {
                            self.lexer.advance();
                            let statements = self.parse_block()?;
                            Box::new(Stmt::Block(statements))
                        }
                        _ => match self.next_stmt()? {
                            Some(s) => Box::new(s),
                            None => Err(ParseError::EmptyWhileStatement {
                                line: self.current_line(),
                            })?,
                        },
                    };
                    return Ok(Some(Stmt::While {
                        condition: cond,
                        body,
                    }));
                }
                TokenKind::Keyword(Keyword::For) => {
                    self.lexer.advance();
                    self.expect_token(TokenKind::LeftParen, "Expect '('.")?;
                    let initializer = match self.lexer.peek().kind {
                        TokenKind::Semicolon => {
                            self.lexer.advance();
                            None
                        }
                        _ => Some(self.next_stmt()?.expect("Expect a statement")),
                    };
                    let cond = self.next_expr()?;
                    self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                    let increment = match self.lexer.peek().kind {
                        TokenKind::RightParen => None,
                        _ => Some(self.next_expr()?),
                    };
                    self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                    let body = match self.lexer.peek().kind {
                        TokenKind::LeftBrace => {
                            self.lexer.advance();
                            let statements = self.parse_block()?;
                            Box::new(Stmt::Block(statements))
                        }
                        _ => match self.next_stmt()? {
                            Some(s) => Box::new(s),
                            // TODO: change to for-related error
                            None => Err(ParseError::EmptyWhileStatement {
                                line: self.current_line(),
                            })?,
                        },
                    };
                    return Ok(Some(Stmt::For {
                        initializer: initializer.map(Box::new),
                        condition: cond,
                        increment,
                        body,
                    }));
                }
                TokenKind::Keyword(Keyword::Fun) => {
                    self.lexer.advance(); // skip over `fun` keyword
                    let ident = self
                        .expect_token(TokenKind::Ident, "Expect identifier.")?
                        .lexeme;
                    self.expect_token(TokenKind::LeftParen, "Expect a '('.")?;
                    let mut params = vec![];
                    while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
                        let t = self.lexer.next_token();
                        match t.kind {
                            TokenKind::Ident => {
                                params.push(t.lexeme);
                                if matches!(self.lexer.peek().kind, TokenKind::Ident) {
                                    return Err(ParseError::MissingCommaBetweenParameters);
                                }
                            }
                            TokenKind::Comma => continue,
                            _ => panic!("Unexpected token"),
                        }
                    }
                    self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                    self.expect_token(TokenKind::LeftBrace, "Expect '{'.")?;
                    let body = self.parse_block()?;
                    return Ok(Some(Stmt::FunDecl {
                        ident,
                        parameters: params,
                        body,
                    }));
                }
                TokenKind::Keyword(Keyword::Return) => {
                    self.lexer.advance();
                    let expr = self.next_expr()?;
                    self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                    return Ok(Some(Stmt::Return(expr)));
                }
                TokenKind::LeftBrace => {
                    self.lexer.advance();
                    return Ok(Some(Stmt::Block(self.parse_block()?)));
                }
                // Consume and ignore comments
                TokenKind::Comment => {
                    self.lexer.advance();
                    continue;
                }
                TokenKind::Eof => return Ok(None),
                // Fallback to expression statement;
                _ => {
                    let expr = self.expr_bp(0)?;
                    self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                    return Ok(Some(Stmt::Expr(expr)));
                }
            }
        }
    }

    pub fn next_expr(&mut self) -> Result<Expr<'src>, ParseError> {
        self.expr_bp(0)
    }

    pub fn expr_bp(&mut self, min_bp: u8) -> Result<Expr<'src>, ParseError> {
        let mut lhs = {
            let token = self.lexer.next_token();
            if let Ok(atom) = Atom::try_from(&token) {
                Expr::Atom(atom)
            } else if token.unterminated() {
                return Err(ParseError::UnterminatedString {
                    line: token.line,
                    lexeme: token.lexeme.to_owned(),
                });
            } else if matches!(token.kind, TokenKind::LeftParen) {
                let members = self.expr_bp(0)?;
                {
                    let t = self.lexer.next_token();
                    if !matches!(t.kind, TokenKind::RightParen) {
                        return Err(ParseError::UnmatchedParen {
                            line: t.line,
                            lexeme: t.lexeme.to_owned(),
                        });
                    }
                }
                Expr::Cons(Op::Group, vec![members])
            } else if let Ok(op) = Op::try_from(&token)
                && let Some(bp) = op.prefix_bp()
            {
                let rhs = self.expr_bp(bp)?;
                Expr::Cons(op, vec![rhs])
            } else {
                return Err(ParseError::UnexpectedToken {
                    line: token.line,
                    lexeme: token.lexeme.to_owned(),
                });
            }
        };

        loop {
            let peeked = self.lexer.peek();
            if let Ok(op) = Op::try_from(peeked) {
                if let Some(l_bp) = op.postfix_bp() {
                    if l_bp < min_bp {
                        break;
                    }
                    self.lexer.advance();
                    lhs = match op {
                        // If a function call, parse all comma-separated arguments first
                        Op::Call => {
                            let mut args = vec![lhs]; // function name should be the first vec member
                            while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
                                args.push(self.next_expr()?);
                                if matches!(self.lexer.peek().kind, TokenKind::Comma) {
                                    self.lexer.advance(); // step over comma
                                    continue;
                                }
                                if args.len() >= 255 {
                                    return Err(ParseError::TooManyArguments);
                                }
                            }
                            self.lexer.advance(); // step over right paren
                            Expr::Cons(Op::Call, args)
                        }
                        // Otherwise treat it like any other postfix operator
                        _ => Expr::Cons(op, vec![lhs]),
                    };
                    // Continue to check the next operator
                    continue;
                }

                if let Some((l_bp, r_bp)) = op.infix_bp() {
                    // If the next operator's left binding power is less than minimum, it is not strong
                    // enough to bind the next atom, so we break and return lhs we've built so far
                    if l_bp < min_bp {
                        break;
                    }
                    self.lexer.advance();
                    let rhs = self.expr_bp(r_bp)?;
                    // What is built so far
                    lhs = Expr::Cons(op, vec![lhs, rhs]);
                    // Continue to check the next operator
                    continue;
                }
            } else if matches!(peeked.kind, TokenKind::RightParen)
                || peeked.is_eof()
                || matches!(peeked.kind, TokenKind::Comma)
            {
                break;
            }
            // else if matches!(peeked.kind, TokenKind::LeftParen) {
            //     // Parse a function call
            //     let mut args = vec![lhs];
            //     while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
            //         args.push(self.next_expr()?)
            //     }
            //     self.expect_token(TokenKind::RightParen)?;
            //     lhs = Expr::Cons(Op::Call, args)
            // }

            break;
        }

        // Return what was built so far as lhs: if nothing was built in a loop above, then it will
        // be an atom found at the beginning of the function. Otherwise it will be constructed
        // expression for this subtree bound by binding power.
        Ok(lhs)
    }

    pub fn current_line(&self) -> usize {
        self.lexer.current_line()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(+ 2.0 3.0)");
    }

    #[test]
    fn add_mult() {
        let mut parser = Parser::new("2 + 3 * 4");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(+ 2.0 (* 3.0 4.0))");
    }

    #[test]
    fn add_mult_add() {
        let mut parser = Parser::new("2 + 3 * 4 + 5");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(+ (+ 2.0 (* 3.0 4.0)) 5.0)");
    }

    #[test]
    fn k_true() {
        let mut parser = Parser::new("true");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "true");
    }

    #[test]
    fn k_false() {
        let mut parser = Parser::new("false");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "false");
    }

    #[test]
    fn nil() {
        let mut parser = Parser::new("nil");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "nil");
    }

    #[test]
    fn number() {
        let mut parser = Parser::new("42.47");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "42.47");

        let mut parser = Parser::new("42");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "42.0");
    }

    #[test]
    fn zero() {
        let mut parser = Parser::new("0.0");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "0.0");
    }

    #[test]
    fn string() {
        let mut parser = Parser::new("\"hello\"");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "hello");
    }

    #[test]
    fn group_single_member() {
        let mut parser = Parser::new("(15)");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(group 15.0)");
    }

    #[test]
    fn group_multiple_members() {
        let mut parser = Parser::new("(10.1 + 21.2)");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(group (+ 10.1 21.2))");
    }

    #[test]
    fn bang_bool() {
        let mut parser = Parser::new("!true");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(! true)");
    }

    #[test]
    fn minus_number() {
        let mut parser = Parser::new("-70");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(- 70.0)");
    }

    #[test]
    fn mult_div() {
        let mut parser = Parser::new("16 * 38 / 58");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(/ (* 16.0 38.0) 58.0)");
    }

    #[test]
    fn mix_expression() {
        let mut parser = Parser::new("(82 * -27 / (34 * 66))");
        let expr = parser.next_expr().unwrap();
        assert_eq!(
            format!("{expr}"),
            "(group (/ (* 82.0 (- 27.0)) (group (* 34.0 66.0))))"
        );
    }

    #[test]
    fn less() {
        let mut parser = Parser::new("83 < 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(< 83.0 99.0)");
    }

    #[test]
    fn less_eq() {
        let mut parser = Parser::new("83 <= 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(<= 83.0 99.0)");
    }

    #[test]
    fn greater() {
        let mut parser = Parser::new("83 > 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(> 83.0 99.0)");
    }

    #[test]
    fn greater_eq() {
        let mut parser = Parser::new("83 >= 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(>= 83.0 99.0)");
    }

    #[test]
    fn equal() {
        let mut parser = Parser::new("83 == 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(== 83.0 99.0)");
    }

    #[test]
    fn not_equal() {
        let mut parser = Parser::new("83 != 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(!= 83.0 99.0)");
    }

    #[test]
    fn assignment() {
        let mut parser = Parser::new("a = 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(= a 99.0)");
    }

    #[test]
    fn sequential_assignments() {
        let mut parser = Parser::new("a = b = 99");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(= a (= b 99.0))");
    }

    #[test]
    // #[should_panic(expected = "[line 1] Error at ')'. Expect expression.")]
    fn error_at_unexpected_token() {
        let mut parser = Parser::new("(72 +)");
        let res = parser.next_expr();
        assert_eq!(
            res,
            Err(ParseError::UnexpectedToken {
                line: 1,
                lexeme: ")".to_owned()
            })
        )
    }

    #[test]
    fn parses_ident_in_expr() {
        let mut parser = Parser::new("a + b");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(+ a b)");
    }

    #[test]
    fn and() {
        let mut parser = Parser::new("true and false");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(and true false)");
    }

    #[test]
    fn or() {
        let mut parser = Parser::new("true or false");
        let expr = parser.next_expr().unwrap();
        assert_eq!(format!("{expr}"), "(or true false)");
    }

    #[test]
    fn error_at_unmatched_paren() {
        let mut parser = Parser::new("(72 + 15");
        let res = parser.next_expr();
        assert_eq!(
            res,
            Err(ParseError::UnmatchedParen {
                line: 1,
                lexeme: "".to_owned()
            })
        )
    }

    #[test]
    fn error_on_unterminated_string() {
        let mut parser = Parser::new("\"hello");
        let res = parser.next_expr();
        assert_eq!(
            res,
            Err(ParseError::UnterminatedString {
                line: 1,
                lexeme: "\"hello".to_owned()
            })
        )
    }

    mod statements {
        use super::*;

        #[test]
        fn parses_print_statement() {
            let mut parser = Parser::new("print \"hello world\";");
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Print(Expr::Atom(Atom::String("hello world"))))
            )
        }

        #[test]
        fn parses_expression_statement() {
            let mut parser = Parser::new("\"hello world\";");
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Expr(Expr::Atom(Atom::String("hello world"))))
            )
        }

        #[test]
        fn parses_var_declaration() {
            let mut parser = Parser::new("var a = 12;");
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Var {
                    name: "a",
                    init_val: Expr::Atom(Atom::Number(12.0)),
                })
            )
        }

        #[test]
        fn parses_uninitialized_var_declarations_with_nil_as_init_value() {
            let mut parser = Parser::new("var a;");
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Var {
                    name: "a",
                    init_val: Expr::Atom(Atom::Nil),
                })
            )
        }

        #[test]
        fn block() {
            let src = "
                {
                    var x = 1; 
                    var y = 2;
                }
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Block(vec![
                    Stmt::Var {
                        name: "x",
                        init_val: Expr::Atom(Atom::Number(1.0))
                    },
                    Stmt::Var {
                        name: "y",
                        init_val: Expr::Atom(Atom::Number(2.0))
                    },
                ]))
            )
        }

        #[test]
        fn two_blocks() {
            let src = "
                {
                    var x = 1; 
                    var y = 2;
                }
                {
                    var a = 3;
                    var b = 4;
                }
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Block(vec![
                    Stmt::Var {
                        name: "x",
                        init_val: Expr::Atom(Atom::Number(1.0))
                    },
                    Stmt::Var {
                        name: "y",
                        init_val: Expr::Atom(Atom::Number(2.0))
                    },
                ]))
            );
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::Block(vec![
                    Stmt::Var {
                        name: "a",
                        init_val: Expr::Atom(Atom::Number(3.0))
                    },
                    Stmt::Var {
                        name: "b",
                        init_val: Expr::Atom(Atom::Number(4.0))
                    },
                ]))
            );
        }

        #[test]
        fn unclosed_block_raises_error() {
            let src = "
                {
                    var bar = 11;
                    var world = 11;
                    {
                        print bar + world;
                }
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt();
            assert_eq!(res, Err(ParseError::UnclosedBlock { line: 8 }));
        }

        #[test]
        fn if_with_inline_statement() {
            let src = "if (true) print \"good\";";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::If {
                    condition: Expr::Atom(Atom::Bool(true)),
                    stmt: Box::new(Stmt::Print(Expr::Atom(Atom::String("good")))),
                    else_stmt: None,
                })
            )
        }

        #[test]
        fn if_with_block_statement() {
            let src = "
                if (true) {
                    var a = 10;
                    print a;
                }
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::If {
                    condition: Expr::Atom(Atom::Bool(true)),
                    stmt: Box::new(Stmt::Block(vec![
                        Stmt::Var {
                            name: "a",
                            init_val: Expr::Atom(Atom::Number(10.0))
                        },
                        Stmt::Print(Expr::Atom(Atom::Ident("a"))),
                    ])),
                    else_stmt: None,
                })
            )
        }

        #[test]
        fn if_with_inline_else_statment() {
            let src = "
                if (false) {
                    print \"good\";
                } else print \"bad\";
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::If {
                    condition: Expr::Atom(Atom::Bool(false)),
                    stmt: Box::new(Stmt::Block(vec![Stmt::Print(Expr::Atom(Atom::String(
                        "good"
                    ))),])),
                    else_stmt: Some(Box::new(Stmt::Print(Expr::Atom(Atom::String("bad")))))
                })
            )
        }

        #[test]
        fn if_with_else_block_statement() {
            let src = "
                if (true) {
                    var a = 10;
                    print a;
                } else {
                    var b = 20;
                    print b;
                }
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::If {
                    condition: Expr::Atom(Atom::Bool(true)),
                    stmt: Box::new(Stmt::Block(vec![
                        Stmt::Var {
                            name: "a",
                            init_val: Expr::Atom(Atom::Number(10.0))
                        },
                        Stmt::Print(Expr::Atom(Atom::Ident("a"))),
                    ])),
                    else_stmt: Some(Box::new(Stmt::Block(vec![
                        Stmt::Var {
                            name: "b",
                            init_val: Expr::Atom(Atom::Number(20.0))
                        },
                        Stmt::Print(Expr::Atom(Atom::Ident("b"))),
                    ]))),
                })
            )
        }

        #[test]
        fn parses_while_stmt_with_block_body() {
            let src = "
                while (true) {
                    print 10;
                }
            ";
            let mut parser = Parser::new(src);
            let res = parser.next_stmt().unwrap();
            assert_eq!(
                res,
                Some(Stmt::While {
                    condition: Expr::Atom(Atom::Bool(true)),
                    body: Box::new(Stmt::Block(vec![Stmt::Print(Expr::Atom(Atom::Number(
                        10.0
                    )))])),
                })
            )
        }

        mod for_stmt {
            use super::*;

            #[test]
            fn parses_stmt_with_single_stmt_body() {
                let src = "
                for (var i = 0; i < 10; i = i + 1) print i;
            ";
                let mut parser = Parser::new(src);
                let res = parser.next_stmt().unwrap();
                assert_eq!(
                    res,
                    Some(Stmt::For {
                        initializer: Some(Box::new(Stmt::Var {
                            name: "i",
                            init_val: Expr::Atom(Atom::Number(0.0)),
                        })),
                        condition: Expr::Cons(
                            Op::Less,
                            vec![Expr::Atom(Atom::Ident("i")), Expr::Atom(Atom::Number(10.0))]
                        ),
                        increment: Some(Expr::Cons(
                            Op::Assign,
                            vec![
                                Expr::Atom(Atom::Ident("i")),
                                Expr::Cons(
                                    Op::Add,
                                    vec![
                                        Expr::Atom(Atom::Ident("i")),
                                        Expr::Atom(Atom::Number(1.0))
                                    ]
                                )
                            ]
                        )),
                        body: Box::new(Stmt::Print(Expr::Atom(Atom::Ident("i")))),
                    })
                )
            }

            #[test]
            fn parses_stmt_with_block_stmt_body() {
                let src = "
                for (var i = 0; i < 10; i = i + 1) {
                    print i;
                }
            ";
                let mut parser = Parser::new(src);
                let res = parser.next_stmt().unwrap();
                assert_eq!(
                    res,
                    Some(Stmt::For {
                        initializer: Some(Box::new(Stmt::Var {
                            name: "i",
                            init_val: Expr::Atom(Atom::Number(0.0)),
                        })),
                        condition: Expr::Cons(
                            Op::Less,
                            vec![Expr::Atom(Atom::Ident("i")), Expr::Atom(Atom::Number(10.0))]
                        ),
                        increment: Some(Expr::Cons(
                            Op::Assign,
                            vec![
                                Expr::Atom(Atom::Ident("i")),
                                Expr::Cons(
                                    Op::Add,
                                    vec![
                                        Expr::Atom(Atom::Ident("i")),
                                        Expr::Atom(Atom::Number(1.0))
                                    ]
                                )
                            ]
                        )),
                        body: Box::new(Stmt::Block(vec![Stmt::Print(Expr::Atom(Atom::Ident(
                            "i"
                        )))])),
                    })
                )
            }

            #[test]
            fn can_skip_increment_expr() {
                let src = "
                    for (var i = 0; i < 3;) {
                        print i;
                        i = i + 1;
                    }
                ";
                let mut parser = Parser::new(src);
                let res = parser.next_stmt().unwrap();
                assert_eq!(
                    res,
                    Some(Stmt::For {
                        initializer: Some(Box::new(Stmt::Var {
                            name: "i",
                            init_val: Expr::Atom(Atom::Number(0.0)),
                        })),
                        condition: Expr::Cons(
                            Op::Less,
                            vec![Expr::Atom(Atom::Ident("i")), Expr::Atom(Atom::Number(3.0))]
                        ),
                        increment: None,
                        body: Box::new(Stmt::Block(vec![
                            Stmt::Print(Expr::Atom(Atom::Ident("i"))),
                            Stmt::Expr(Expr::Cons(
                                Op::Assign,
                                vec![
                                    Expr::Atom(Atom::Ident("i")),
                                    Expr::Cons(
                                        Op::Add,
                                        vec![
                                            Expr::Atom(Atom::Ident("i")),
                                            Expr::Atom(Atom::Number(1.0))
                                        ]
                                    )
                                ]
                            ))
                        ])),
                    })
                )
            }

            #[test]
            fn can_skip_initializer() {
                let src = "
                    for (; i < 3;) {
                        print i;
                    }
                ";
                let mut parser = Parser::new(src);
                let res = parser.next_stmt().unwrap();
                assert_eq!(
                    res,
                    Some(Stmt::For {
                        initializer: None,
                        condition: Expr::Cons(
                            Op::Less,
                            vec![Expr::Atom(Atom::Ident("i")), Expr::Atom(Atom::Number(3.0))]
                        ),
                        increment: None,
                        body: Box::new(Stmt::Block(vec![Stmt::Print(Expr::Atom(Atom::Ident(
                            "i"
                        ))),])),
                    })
                )
            }
        }

        mod function_call {
            use super::*;

            #[test]
            fn simple_call() {
                let src = "fn_name()";
                let mut parser = Parser::new(src);
                let res = parser.next_expr().unwrap();
                assert_eq!(
                    res,
                    Expr::Cons(Op::Call, vec![Expr::Atom(Atom::Ident("fn_name"))])
                )
            }

            #[test]
            fn call_with_argument() {
                let src = "fn_name(10)";
                let mut parser = Parser::new(src);
                let res = parser.next_expr().unwrap();
                assert_eq!(
                    res,
                    Expr::Cons(
                        Op::Call,
                        vec![
                            Expr::Atom(Atom::Ident("fn_name"),),
                            Expr::Atom(Atom::Number(10.0))
                        ]
                    )
                )
            }

            #[test]
            fn call_with_multiple_arguments() {
                let src = "fn_name(10, \"a\", true)";
                let mut parser = Parser::new(src);
                let res = parser.next_expr().unwrap();
                assert_eq!(
                    res,
                    Expr::Cons(
                        Op::Call,
                        vec![
                            Expr::Atom(Atom::Ident("fn_name")),
                            Expr::Atom(Atom::Number(10.0)),
                            Expr::Atom(Atom::String("a")),
                            Expr::Atom(Atom::Bool(true)),
                        ]
                    )
                )
            }
        }
    }

    #[test]
    fn parses_user_defined_func_with_no_parameters() {
        let src = "
            fun my_func() {
                print 10;
            }
        ";
        let mut parser = Parser::new(src);
        let res = parser.next_stmt().unwrap();
        assert_eq!(
            res,
            Some(Stmt::FunDecl {
                ident: "my_func",
                parameters: vec![],
                body: vec![Stmt::Print(Expr::Atom(Atom::Number(10.0)))],
            })
        )
    }

    #[test]
    fn parses_user_defined_func_with_params() {
        let src = "
            fun my_func(a, b, c) {
                print a;
            }
        ";
        let mut parser = Parser::new(src);
        let res = parser.next_stmt().unwrap();
        assert_eq!(
            res,
            Some(Stmt::FunDecl {
                ident: "my_func",
                parameters: vec!["a", "b", "c"],
                body: vec![Stmt::Print(Expr::Atom(Atom::Ident("a")))],
            })
        )
    }

    #[test]
    fn parses_return_statement() {
        let src = "
            fun my_func() {
                return 10;
            }
        ";
        let mut parser = Parser::new(src);
        let res = parser.next_stmt().unwrap();
        assert_eq!(
            res,
            Some(Stmt::FunDecl {
                ident: "my_func",
                parameters: vec![],
                body: vec![Stmt::Return(Expr::Atom(Atom::Number(10.0)))],
            })
        )
    }
}
