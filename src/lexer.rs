use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'src> {
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
    Minus,
    /// `+`
    Plus,
    /// `;`
    Semicolon,
    /// `*`
    Star,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `!`
    Bang,
    /// `!=`
    BangEq,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `/`
    Slash,
    /// `// comment`
    Comment,
    /// e.g. `foo`, `bar`, `_baz`
    Identifier,
    /// Any whitespace char
    Whitespace,
    Literal(Literal<'src>),
    Keyword(Keyword),
    Unknown {
        ch: char,
        line: usize,
    },
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'src> {
    String { s: &'src str, terminated: bool },
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Var,
    Fun,
    Return,
    Class,
    This,
    Super,
    Nil,
    If,
    Else,
    And,
    Or,
    True,
    False,
    For,
    While,
    Print,
}

#[derive(Debug, PartialEq)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub lexeme: &'src str,
    pub len: usize,
    pub line: usize,
}

impl<'src> Token<'src> {
    fn new(kind: TokenKind<'src>, lexeme: &'src str, len: usize, line: usize) -> Self {
        Self {
            kind,
            lexeme,
            len,
            line,
        }
    }

    fn eof(line: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            lexeme: "",
            len: 0,
            line,
        }
    }

    pub fn unterminated(&self) -> bool {
        match &self.kind {
            TokenKind::Literal(Literal::String { terminated, .. }) => !*terminated,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }
}

struct Cursor<'a> {
    /// Iterator over src chars
    chars: Peekable<Chars<'a>>,
    /// Current line
    line: usize,
    byte_pos: usize,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: chars.peekable(),
            line: 1,
            byte_pos: 0,
        }
    }

    /// Returns next char and moves the cursor
    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        if ch == '\n' {
            self.line += 1;
        }
        self.byte_pos += ch.len_utf8();
        Some(ch)
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
    peeked: Option<Token<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            cursor: Cursor::new(src.chars()),
            peeked: None,
        }
    }

    pub fn next_token(&mut self) -> Token<'src> {
        // Return a buffered token if present, otherwise continue with the regular execution
        if let Some(token) = self.peeked.take() {
            return token;
        }

        // Skip all whitespace chars
        while self.cursor.peek().is_whitespace() {
            self.cursor.bump();
        }

        let start = self.cursor.byte_pos;
        let Some(ch) = self.cursor.bump() else {
            return Token::eof(self.cursor.line);
        };

        let token_kind = match ch {
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
                    TokenKind::EqEq
                }
                _ => TokenKind::Eq,
            },
            '!' => match self.cursor.peek() {
                '=' => {
                    self.cursor.bump();
                    TokenKind::BangEq
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
                self.cursor.eat_while(|v| v != '"' && v != EOF_CHAR);
                if self.cursor.peek() == EOF_CHAR {
                    TokenKind::Literal(Literal::String {
                        s: "",
                        terminated: false,
                    })
                } else {
                    self.cursor.bump();
                    TokenKind::Literal(Literal::String {
                        s: self.src[start..self.cursor.byte_pos].trim_matches(|v| v == '"'),
                        terminated: true,
                    })
                }
            }
            '0'..='9' => {
                self.cursor.eat_while(|v| v.is_ascii_digit() || v == '.');
                let v: f64 = self.src[start..self.cursor.byte_pos].parse().unwrap();
                TokenKind::Literal(Literal::Number(v))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.cursor.eat_while(|v| v.is_alphanumeric() || v == '_');
                let v = &self.src[start..self.cursor.byte_pos];
                match v {
                    "var" => TokenKind::Keyword(Keyword::Var),
                    "fun" => TokenKind::Keyword(Keyword::Fun),
                    "return" => TokenKind::Keyword(Keyword::Return),
                    "class" => TokenKind::Keyword(Keyword::Class),
                    "this" => TokenKind::Keyword(Keyword::This),
                    "super" => TokenKind::Keyword(Keyword::Super),
                    "nil" => TokenKind::Keyword(Keyword::Nil),
                    "if" => TokenKind::Keyword(Keyword::If),
                    "else" => TokenKind::Keyword(Keyword::Else),
                    "and" => TokenKind::Keyword(Keyword::And),
                    "or" => TokenKind::Keyword(Keyword::Or),
                    "true" => TokenKind::Keyword(Keyword::True),
                    "false" => TokenKind::Keyword(Keyword::False),
                    "for" => TokenKind::Keyword(Keyword::For),
                    "while" => TokenKind::Keyword(Keyword::While),
                    "print" => TokenKind::Keyword(Keyword::Print),
                    _ => TokenKind::Identifier,
                }
            }
            c => TokenKind::Unknown {
                ch: c,
                line: self.cursor.line,
            },
        };

        let lexeme = &self.src[start..self.cursor.byte_pos];
        Token::new(token_kind, lexeme, lexeme.chars().count(), self.cursor.line)
    }

    /// Peek into the next token. Peeked token is cached under the hood and will be returned in
    /// `next_token()` call.
    pub fn peek(&mut self) -> &Token<'src> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next_token());
        }
        self.peeked.as_ref().unwrap()
    }

    /// Convenience method around `next_token()` to consume the next token and advance cursor.
    pub fn advance(&mut self) {
        self.next_token();
    }

    pub fn tokenize(&mut self) -> Vec<Token<'src>> {
        self.by_ref().collect()
    }

    pub fn current_line(&self) -> usize {
        self.cursor.line
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
                .is_some_and(|v| matches!(v.kind, TokenKind::Eq))
        );
    }

    #[test]
    fn equal_equal() {
        let mut lexer = Lexer::new("==");
        assert!(
            lexer
                .next()
                .is_some_and(|v| matches!(v.kind, TokenKind::EqEq))
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
                .is_some_and(|v| matches!(v.kind, TokenKind::BangEq))
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
                kind: TokenKind::Literal(Literal::String {
                    s: "this is a test string",
                    terminated: true
                }),
                lexeme: "\"this is a test string\"",
                len: 23,
                line: 1,
            }]
        )
    }

    #[test]
    fn unterminaed_string() {
        let mut lexer = Lexer::new(r#""this is an unterminated string"#);
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Literal(Literal::String {
                    s: "",
                    terminated: false
                }),
                lexeme: "\"this is an unterminated string",
                len: 31,
                line: 1,
            }]
        )
    }

    mod numbers {
        use super::*;

        #[test]
        fn integer() {
            let mut lexer = Lexer::new("1234");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Literal(Literal::Number(1234.0)),
                    lexeme: "1234",
                    len: 4,
                    line: 1,
                }]
            )
        }

        #[test]
        fn integer_starts_with_9() {
            let mut lexer = Lexer::new("9123");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Literal(Literal::Number(9123.0)),
                    lexeme: "9123",
                    len: 4,
                    line: 1,
                }]
            )
        }

        #[test]
        fn float() {
            let mut lexer = Lexer::new("1234.32");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Literal(Literal::Number(1234.32)),
                    lexeme: "1234.32",
                    len: 7,
                    line: 1,
                }]
            )
        }
    }

    mod identifiers {
        use super::*;

        #[test]
        fn simple() {
            let mut lexer = Lexer::new("foo");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Identifier,
                    lexeme: "foo",
                    len: 3,
                    line: 1,
                }]
            )
        }

        #[test]
        fn leading_underscore() {
            let mut lexer = Lexer::new("_foo");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Identifier,
                    lexeme: "_foo",
                    len: 4,
                    line: 1,
                }]
            )
        }
    }

    mod keywords {
        use super::*;

        #[test]
        fn var() {
            let mut lexer = Lexer::new("var");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Var),
                    lexeme: "var",
                    len: 3,
                    line: 1,
                }]
            )
        }

        #[test]
        fn fun() {
            let mut lexer = Lexer::new("fun");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Fun),
                    lexeme: "fun",
                    len: 3,
                    line: 1,
                }]
            )
        }
        #[test]
        fn k_return() {
            let mut lexer = Lexer::new("return");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Return),
                    lexeme: "return",
                    len: 6,
                    line: 1,
                }]
            )
        }
        #[test]
        fn class() {
            let mut lexer = Lexer::new("class");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Class),
                    lexeme: "class",
                    len: 5,
                    line: 1,
                }]
            )
        }
        #[test]
        fn this() {
            let mut lexer = Lexer::new("this");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::This),
                    lexeme: "this",
                    len: 4,
                    line: 1,
                }]
            )
        }
        #[test]
        fn k_super() {
            let mut lexer = Lexer::new("super");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Super),
                    lexeme: "super",
                    len: 5,
                    line: 1,
                }]
            )
        }
        #[test]
        fn nil() {
            let mut lexer = Lexer::new("nil");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Nil),
                    lexeme: "nil",
                    len: 3,
                    line: 1,
                }]
            )
        }
        #[test]
        fn k_if() {
            let mut lexer = Lexer::new("if");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::If),
                    lexeme: "if",
                    len: 2,
                    line: 1,
                }]
            )
        }
        #[test]
        fn k_else() {
            let mut lexer = Lexer::new("else");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Else),
                    lexeme: "else",
                    len: 4,
                    line: 1,
                }]
            )
        }
        #[test]
        fn and() {
            let mut lexer = Lexer::new("and");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::And),
                    lexeme: "and",
                    len: 3,
                    line: 1,
                }]
            )
        }
        #[test]
        fn or() {
            let mut lexer = Lexer::new("or");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Or),
                    lexeme: "or",
                    len: 2,
                    line: 1,
                }]
            )
        }
        #[test]
        fn k_true() {
            let mut lexer = Lexer::new("true");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::True),
                    lexeme: "true",
                    len: 4,
                    line: 1,
                }]
            )
        }
        #[test]
        fn k_false() {
            let mut lexer = Lexer::new("false");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::False),
                    lexeme: "false",
                    len: 5,
                    line: 1,
                }]
            )
        }

        #[test]
        fn k_for() {
            let mut lexer = Lexer::new("for");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::For),
                    lexeme: "for",
                    len: 3,
                    line: 1,
                }]
            )
        }

        #[test]
        fn k_while() {
            let mut lexer = Lexer::new("while");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::While),
                    lexeme: "while",
                    len: 5,
                    line: 1,
                }]
            )
        }

        #[test]
        fn print() {
            let mut lexer = Lexer::new("print");
            let tokens = lexer.tokenize();
            assert_eq!(
                tokens,
                vec![Token {
                    kind: TokenKind::Keyword(Keyword::Print),
                    lexeme: "print",
                    len: 5,
                    line: 1,
                }]
            )
        }
    }
}
