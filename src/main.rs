#![allow(unused_variables)]
use std::env;
use std::fs;

use codecrafters_interpreter::lexer::Lexer;
use codecrafters_interpreter::lexer::TokenKind;

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

            let mut lexer = Lexer::new(&file_contents);
            loop {
                let token = lexer.next_token();

                format!("{token}");

                if matches!(token.kind, TokenKind::Eof) {
                    break;
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
