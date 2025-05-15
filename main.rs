use std::io::{self, Write};

mod tokenizer;
mod parser;
mod expression;

use tokenizer::tokenize;
use expression::parse_expression;

fn main() {
    println!("Enter a SQL query or expression:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let tokens = tokenize(&input);
    println!("\nTokenized output as Rust vector:");
    println!("vec![");
    for token in &tokens {
        println!("    {:?},", token);
    }
    println!("]");

    println!("\nParsed expression (if any):");
    match parse_expression(&tokens, 0) {
        Ok((expr, _)) => println!("{:#?}", expr),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\nParsed statement (if any):");
    match parser::parse(&tokens) {
        Ok(stmt) => println!("{:#?}", stmt),
        Err(e) => eprintln!("Error: {}", e),
    }
}
