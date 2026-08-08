use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Whitespace,
    Unknown { ch: char, line: usize },
    Eof,
}

pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, len: usize) -> Self {
        Self { kind, len }
    }

    fn eof() -> Self {
        Self {
            kind: TokenKind::Eof,
            len: 0,
        }
    }
}

struct Cursor<'a> {
    /// Iterator over src chars
    chars: Peekable<Chars<'a>>,
    cur_line: usize,
}

impl<'a> Cursor<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: chars.peekable(),
            cur_line: 1,
        }
    }

    /// Returns next char and moves the cursor
    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if ch == Some('\n') {
            self.cur_line += 1;
        }
        ch
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}

pub struct Lexer<'src> {
    #[allow(dead_code)]
    src: &'src str,
    cursor: Cursor<'src>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            cursor: Cursor::new(src.chars()),
        }
    }

    pub fn next_token(&mut self) -> Token {
        let Some(ch) = self.cursor.bump() else {
            return Token::eof();
        };

        let mut len = 1;
        let token_kind = match ch {
            c if c.is_whitespace() => {
                while self.cursor.peek().is_some_and(|v| v.is_whitespace()) {
                    self.cursor.bump();
                    len += 1;
                }
                TokenKind::Whitespace
            }
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '*' => TokenKind::Star,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            ';' => TokenKind::Semicolon,
            '=' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::EqualEqual
                }
                _ => TokenKind::Equal,
            },
            '!' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::BangEqual
                }
                _ => TokenKind::Bang,
            },
            '<' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::LessEqual
                }
                _ => TokenKind::Less,
            },
            '>' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::GreaterEqual
                }
                _ => TokenKind::Greater,
            },
            c => TokenKind::Unknown {
                ch: c,
                line: self.cursor.cur_line,
            },
        };

        Token::new(token_kind, len)
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        match token.kind {
            TokenKind::Eof => None,
            _ => Some(token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parens() {
        let mut lexer = Lexer::new("()");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::LeftParen))
        );
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::RightParen))
        );
    }

    #[test]
    fn braces() {
        let mut lexer = Lexer::new("{}");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::LeftBrace))
        );
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::RightBrace))
        );
    }

    #[test]
    fn comma() {
        let mut lexer = Lexer::new(",");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Comma))
        );
    }

    #[test]
    fn dot() {
        let mut lexer = Lexer::new(".");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Dot))
        );
    }

    #[test]
    fn minus() {
        let mut lexer = Lexer::new("-");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Minus))
        );
    }

    #[test]
    fn plus() {
        let mut lexer = Lexer::new("+");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Plus))
        );
    }

    #[test]
    fn star() {
        let mut lexer = Lexer::new("*");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Star))
        );
    }

    #[test]
    fn semicolon() {
        let mut lexer = Lexer::new(";");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Semicolon))
        );
    }

    #[test]
    fn iterates_over_all_tokens() {
        let mut lexer = Lexer::new("({})");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::LeftParen))
        );
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::LeftBrace))
        );
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::RightBrace))
        );
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::RightParen))
        );
    }

    #[test]
    fn returns_unknown_token_kind_for_unexpected_token() {
        let mut lexer = Lexer::new("$");
        let ch = lexer.next().unwrap();
        assert!(matches!(ch.kind, TokenKind::Unknown { .. }));
    }

    #[test]
    fn returns_unknown_token_line() {
        let mut lexer = Lexer::new("$");
        let ch = lexer.next().unwrap();
        assert!(matches!(ch.kind, TokenKind::Unknown { line: 1, .. }));
    }

    #[test]
    fn equal() {
        let mut lexer = Lexer::new("=");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Equal))
        );
    }

    #[test]
    fn equal_equal() {
        let mut lexer = Lexer::new("==");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::EqualEqual))
        );
    }

    #[test]
    fn bang() {
        let mut lexer = Lexer::new("!");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Bang))
        );
    }

    #[test]
    fn bang_equal() {
        let mut lexer = Lexer::new("!=");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::BangEqual))
        );
    }

    #[test]
    fn less() {
        let mut lexer = Lexer::new("<");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Less))
        );
    }

    #[test]
    fn less_equal() {
        let mut lexer = Lexer::new("<=");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::LessEqual))
        );
    }

    #[test]
    fn greater() {
        let mut lexer = Lexer::new(">");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Greater))
        );
    }

    #[test]
    fn greater_equal() {
        let mut lexer = Lexer::new(">=");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::GreaterEqual))
        );
    }
}
