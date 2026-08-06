use std::{fmt::Display, iter::Peekable, str::Chars};

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
    Whitespace,
    Unknown,
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
}

impl<'a> Cursor<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: chars.peekable(),
        }
    }

    /// Returns next char and moves the cursor
    fn bump(&mut self) -> Option<char> {
        self.chars.next()
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

        let token_kind = match ch {
            c if c.is_whitespace() => {
                while self.cursor.peek().is_some_and(|v| v.is_whitespace()) {
                    self.cursor.bump();
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
            _ => TokenKind::Unknown,
        };

        Token::new(token_kind, 0)
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
}
