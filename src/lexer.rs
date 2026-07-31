use std::{fmt::Display, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `-`
    Dash,
    /// `+`
    Plus,
    /// `;`
    Semicolon,
    /// `*`
    Star,
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
            TokenKind::LeftBrace => f.write_str("LEFT_BRACE { null"),
            TokenKind::RightBrace => f.write_str("RIGHT_BRACE } null"),
            TokenKind::Comma => f.write_str("COMMA } null"),
            TokenKind::Dot => f.write_str("DOT } null"),
            TokenKind::Dash => f.write_str("DASH } null"),
            TokenKind::Plus => f.write_str("PLUS } null"),
            TokenKind::Semicolon => f.write_str("SEMICOLON } null"),
            TokenKind::Star => f.write_str("STAR } null"),
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
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
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
    fn lex_left_brace() {
        let mut lexer = Lexer::new("{");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::LeftBrace);
    }

    #[test]
    fn lex_right_brace() {
        let mut lexer = Lexer::new("}");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::RightBrace);
    }

    #[test]
    fn lex_comma() {
        let mut lexer = Lexer::new(",");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Comma);
    }

    #[test]
    fn lex_dot() {
        let mut lexer = Lexer::new(".");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Dot);
    }

    #[test]
    fn lex_dash() {
        let mut lexer = Lexer::new("-");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Dash);
    }

    #[test]
    fn lex_plus() {
        let mut lexer = Lexer::new("+");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Plus);
    }

    #[test]
    fn lex_semicolon() {
        let mut lexer = Lexer::new(";");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Semicolon);
    }

    #[test]
    fn lex_star() {
        let mut lexer = Lexer::new("*");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Star);
    }

    #[test]
    fn empty_string_produces_eof_token() {
        let mut lexer = Lexer::new("");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Eof);
    }
}
