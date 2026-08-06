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
                if let Some(s) = format_token(&token) {
                    println!("{s}");
                }
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

fn format_token(t: &Token) -> Option<String> {
    match t.kind {
        TokenKind::LeftParen => Some("LEFT_PAREN ( null".to_string()),
        TokenKind::RightParen => Some("RIGHT_PAREN ) null".to_string()),
        TokenKind::LeftBrace => Some("LEFT_BRACE { null".to_string()),
        TokenKind::RightBrace => Some("RIGHT_BRACE } null".to_string()),
        TokenKind::Comma => Some("COMMA , null".to_string()),
        TokenKind::Dot => Some("DOT . null".to_string()),
        TokenKind::Minus => Some("MINUS - null".to_string()),
        TokenKind::Plus => Some("PLUS + null".to_string()),
        TokenKind::Semicolon => Some("SEMICOLON ; null".to_string()),
        TokenKind::Star => Some("STAR * null".to_string()),
        TokenKind::Unknown { ch, line } => {
            Some(format!("[line {line}] Unexpected character: {ch}"))
        }
        TokenKind::Whitespace | TokenKind::Eof => None,
    }
}
