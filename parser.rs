use crate::tokenizer::{Token, Token::*};
use crate::expression::Expression;

#[derive(Debug)]
pub enum Statement {
    Select {
        columns: Vec<Expression>,
        from: String,
        r#where: Option<Expression>,
        orderby: Vec<String>,
    },
    CreateTable {
        name: String,
    },
}

pub fn parse(tokens: &[Token]) -> Result<Statement, String> {
    let mut iter = tokens.iter().peekable();

    match iter.next() {
        Some(Keyword(k)) if k == "SELECT" => {
            let mut columns = vec![];

            loop {
                match iter.next() {
                    Some(Identifier(name)) => columns.push(Expression::Identifier(name.to_string())),
                    Some(StringLiteral(s)) => columns.push(Expression::String(s.to_string())),
                    Some(Star) => columns.push(Expression::Identifier("*".to_string())),
                    Some(Comma) => continue,
                    Some(Keyword(k)) if k == "FROM" => break,
                    _ => return Err("Unexpected token in SELECT".into()),
                }
            }

            let from = match iter.next() {
                Some(Identifier(name)) => name.to_string(),
                _ => return Err("Expected table name after FROM".into()),
            };

            Ok(Statement::Select {
                columns,
                from,
                r#where: None,
                orderby: vec![],
            })
        }

        Some(Keyword(k)) if k == "CREATE" => {
            if let Some(Keyword(k)) = iter.next() {
                if k == "TABLE" {
                    if let Some(Identifier(name)) = iter.next() {
                        return Ok(Statement::CreateTable {
                            name: name.to_string(),
                        });
                    }
                }
            }
            Err("Invalid CREATE TABLE syntax".into())
        }

        _ => Err("Unsupported or invalid SQL statement".into()),
    }
}
