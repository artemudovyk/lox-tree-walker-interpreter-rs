use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'src> {
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
    Slash,
    Comment,
    Whitespace,
    Literal(Literal<'src>),
    Unknown { ch: char, line: usize },
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal<'src> {
    String(&'src str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub lexeme: &'src str,
    pub len: usize,
}

impl<'src> Token<'src> {
    fn new(kind: TokenKind<'src>, lexeme: &'src str, len: usize) -> Self {
        Self { kind, lexeme, len }
    }

    fn eof() -> Self {
        Self {
            kind: TokenKind::Eof,
            lexeme: "",
            len: 0,
        }
    }
}

struct Cursor<'a> {
    /// Iterator over src chars
    chars: Peekable<Chars<'a>>,
    /// Current line
    line: usize,
    /// Consumed chars
    pos: usize,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: chars.peekable(),
            line: 1,
            pos: 0,
        }
    }

    /// Returns next char and moves the cursor
    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if ch == Some('\n') {
            self.line += 1;
        }
        self.pos += 1;
        ch
    }

    fn peek(&mut self) -> char {
        self.chars.peek().copied().unwrap_or(EOF_CHAR)
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) {
            self.bump();
        }
    }
}

pub struct Lexer<'src> {
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

    pub fn next_token(&mut self) -> Token<'src> {
        let pos = self.cursor.pos;
        let Some(ch) = self.cursor.bump() else {
            return Token::eof();
        };

        let token_kind = match ch {
            c if c.is_whitespace() => {
                while self.cursor.peek().is_whitespace() {
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
            '=' => match self.cursor.peek() {
                '=' => {
                    self.cursor.bump();
                    TokenKind::EqualEqual
                }
                _ => TokenKind::Equal,
            },
            '!' => match self.cursor.peek() {
                '=' => {
                    self.cursor.bump();
                    TokenKind::BangEqual
                }
                _ => TokenKind::Bang,
            },
            '<' => match self.cursor.peek() {
                '=' => {
                    self.cursor.bump();
                    TokenKind::LessEqual
                }
                _ => TokenKind::Less,
            },
            '>' => match self.cursor.peek() {
                '=' => {
                    self.cursor.bump();
                    TokenKind::GreaterEqual
                }
                _ => TokenKind::Greater,
            },
            '/' => match self.cursor.peek() {
                '/' => {
                    self.cursor.eat_while(|v| v != '\n' && v != EOF_CHAR);
                    TokenKind::Comment
                }
                _ => TokenKind::Slash,
            },
            '"' => {
                self.cursor.eat_while(|v| v != '"');
                self.cursor.bump();
                TokenKind::Literal(Literal::String(&self.src[pos + 1..self.cursor.pos - 1]))
            }
            c => TokenKind::Unknown {
                ch: c,
                line: self.cursor.line,
            },
        };

        Token::new(
            token_kind,
            // TODO: will probably fail if there are chars that take more than 1 byte
            &self.src[pos..self.cursor.pos],
            self.cursor.pos - pos + 2,
        )
    }

    pub fn tokenize(&mut self) -> Vec<Token<'src>> {
        self.by_ref().collect()
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token<'src>;

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

    #[test]
    fn slash() {
        let mut lexer = Lexer::new("/");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Slash))
        );
    }

    #[test]
    fn comment() {
        let mut lexer = Lexer::new("// comment");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::Comment))
        );
    }

    #[test]
    fn string() {
        let mut lexer = Lexer::new(r#""this is a test string""#);
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Literal(Literal::String("this is a test string")),
                lexeme: "\"this is a test string\"",
                len: 25,
            }]
        )
    }
}
