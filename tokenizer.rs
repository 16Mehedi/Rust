#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Operator(String),
    Number(i64),
    StringLiteral(String),
    BoolLiteral(bool), // ✅ New token for TRUE/FALSE
    Comma,
    Semicolon,
    LParen,
    RParen,
    Star,
    EOF,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '<' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Operator("<=".to_string()));
                } else {
                    tokens.push(Token::Operator("<".to_string()));
                }
            }
            '>' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Operator(">=".to_string()));
                } else {
                    tokens.push(Token::Operator(">".to_string()));
                }
            }
            '=' => {
                chars.next();
                tokens.push(Token::Operator("=".to_string()));
            }
            '+' | '-' | '/' => {
                let op = chars.next().unwrap();
                tokens.push(Token::Operator(op.to_string()));
            }
            '\'' | '"' => {
                let quote = chars.next().unwrap();
                let mut value = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == quote {
                        break;
                    }
                    value.push(c);
                }
                tokens.push(Token::StringLiteral(value));
            }
            _ => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '.' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let upper = ident.to_uppercase();
                match upper.as_str() {
                    "SELECT" | "FROM" | "WHERE" | "CREATE" | "TABLE" | "ORDER" | "BY" | "NOT" => {
                        tokens.push(Token::Keyword(upper));
                    }
                    "AND" | "OR" => {
                        tokens.push(Token::Operator(upper));
                    }
                    "TRUE" => {
                        tokens.push(Token::BoolLiteral(true));
                    }
                    "FALSE" => {
                        tokens.push(Token::BoolLiteral(false));
                    }
                    _ => {
                        if let Ok(num) = ident.parse::<i64>() {
                            tokens.push(Token::Number(num));
                        } else {
                            tokens.push(Token::Identifier(ident));
                        }
                    }
                }
            }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
