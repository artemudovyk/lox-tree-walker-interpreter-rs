use std::{fmt::Display, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    /// Any whitespace char
    Whitespace,
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
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TokenKind::LeftParen => f.write_str("LEFT_PAREN ( null"),
            TokenKind::RightParen => f.write_str("RIGHT_PAREN ) null"),
            // TODO: remove?
            TokenKind::Whitespace => f.write_str("WHITESPACE  null"),
            TokenKind::Eof => f.write_str("EOF  null"),
        }
    }
}

struct Cursor<'a> {
    /// Iterator over src chars
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self { chars }
    }

    /// Returns next char and moves the cursor
    fn bump(&mut self) -> Option<char> {
        self.chars.next()
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
        let Some(char) = self.cursor.bump() else {
            return Token::new(TokenKind::Eof, 0);
        };

        dbg!(&char);

        let token_kind = match char {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '\n' => TokenKind::Whitespace,
            _ => TokenKind::Eof,
        };

        Token::new(token_kind, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_left_paren() {
        let mut lexer = Lexer::new("(");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::LeftParen);
    }

    #[test]
    fn lex_right_paren() {
        let mut lexer = Lexer::new(")");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::RightParen);
    }

    #[test]
    fn empty_string_produces_eof_token() {
        let mut lexer = Lexer::new("");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Eof);
    }
}
