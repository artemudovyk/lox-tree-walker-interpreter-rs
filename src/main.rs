#![allow(unused_variables)]
use codecrafters_interpreter::lexer::Lexer;
use codecrafters_interpreter::lexer::Token;
use codecrafters_interpreter::lexer::TokenKind;
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
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let lexer = Lexer::new(&file_contents);
            let mut is_valid = true;
            for token in lexer {
                print_token(&token);

                if matches!(token.kind, TokenKind::Unknown { .. }) {
                    is_valid = false;
                }
            }
            println!("EOF  null");

            if !is_valid {
                std::process::exit(65)
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn print_token(t: &Token) {
    match t.kind {
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
        TokenKind::Equal => println!("EQUAL = null"),
        TokenKind::EqualEqual => println!("EQUAL_EQUAL == null"),
        TokenKind::Unknown { ch, line } => {
            eprintln!("[line {line}] Error: Unexpected character: {ch}")
        }
        TokenKind::Whitespace | TokenKind::Eof => {}
    }
}
