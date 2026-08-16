#![allow(unused_variables)]
use codecrafters_interpreter::{
    lexer::{Keyword, Lexer, Literal, Token, TokenKind},
    parser_pratt_v2::Parser,
};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let lexer = Lexer::new(&file_contents);
            let mut is_valid = true;
            for token in lexer {
                print_token(&token);

                if matches!(token.kind, TokenKind::Unknown { .. }) || token.unterminated() {
                    is_valid = false;
                }
            }
            println!("EOF  null");

            if !is_valid {
                std::process::exit(65)
            }
        }
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });
            let mut parser = Parser::new(&file_contents);
            let expr = parser.next_expr();
            println!("{expr}");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn print_token(t: &Token) {
    match &t.kind {
        TokenKind::LeftParen => println!("LEFT_PAREN ( null"),
        TokenKind::RightParen => println!("RIGHT_PAREN ) null"),
        TokenKind::LeftBrace => println!("LEFT_BRACE {{ null"),
        TokenKind::RightBrace => println!("RIGHT_BRACE }} null"),
        TokenKind::Comma => println!("COMMA , null"),
        TokenKind::Dot => println!("DOT . null"),
        TokenKind::Minus => println!("MINUS - null"),
        TokenKind::Plus => println!("PLUS + null"),
        TokenKind::Semicolon => println!("SEMICOLON ; null"),
        TokenKind::Star => println!("STAR * null"),
        TokenKind::Eq => println!("EQUAL = null"),
        TokenKind::EqEq => println!("EQUAL_EQUAL == null"),
        TokenKind::Bang => println!("BANG ! null"),
        TokenKind::BangEq => println!("BANG_EQUAL != null"),
        TokenKind::Less => println!("LESS < null"),
        TokenKind::LessEqual => println!("LESS_EQUAL <= null"),
        TokenKind::Greater => println!("GREATER > null"),
        TokenKind::GreaterEqual => println!("GREATER_EQUAL >= null"),
        TokenKind::Slash => println!("SLASH / null"),
        TokenKind::Literal(lit) => match lit {
            Literal::String { s, terminated } => {
                if *terminated {
                    println!("STRING {} {}", t.lexeme, s)
                } else {
                    eprintln!("[line {}] Error: Unterminated string.", t.line)
                }
            }
            Literal::Number(n) => {
                if *n == n.trunc() {
                    println!("NUMBER {} {:.1}", t.lexeme, n)
                } else {
                    println!("NUMBER {} {}", t.lexeme, n)
                }
            }
        },
        TokenKind::Identifier => println!("IDENTIFIER {} null", t.lexeme),
        TokenKind::Unknown { ch, line } => {
            eprintln!("[line {line}] Error: Unexpected character: {ch}")
        }
        TokenKind::Keyword(keyword) => match keyword {
            Keyword::Var => println!("VAR {} null", t.lexeme),
            Keyword::Fun => println!("FUN {} null", t.lexeme),
            Keyword::Return => println!("RETURN {} null", t.lexeme),
            Keyword::Class => println!("CLASS {} null", t.lexeme),
            Keyword::This => println!("THIS {} null", t.lexeme),
            Keyword::Super => println!("SUPER {} null", t.lexeme),
            Keyword::Nil => println!("NIL {} null", t.lexeme),
            Keyword::If => println!("IF {} null", t.lexeme),
            Keyword::Else => println!("ELSE {} null", t.lexeme),
            Keyword::And => println!("AND {} null", t.lexeme),
            Keyword::Or => println!("OR {} null", t.lexeme),
            Keyword::True => println!("TRUE {} null", t.lexeme),
            Keyword::False => println!("FALSE {} null", t.lexeme),
            Keyword::For => println!("FOR {} null", t.lexeme),
            Keyword::While => println!("WHILE {} null", t.lexeme),
            Keyword::Print => println!("PRINT {} null", t.lexeme),
        },
        TokenKind::Whitespace | TokenKind::Comment | TokenKind::Eof => {}
    }
}
