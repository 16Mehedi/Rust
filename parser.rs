use crate::expression::Expression;
use crate::tokenizer::{Token, Token::*};

#[derive(Debug)]
pub enum DBType {
    Int,
    Varchar(u64),
    Bool,
}

#[derive(Debug)]
pub enum Constraint {
    PrimaryKey,
    NotNull,
    Check(Expression),
}

#[derive(Debug)]
pub struct TableColumn {
    pub column_name: String,
    pub column_type: DBType,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug)]
pub enum Statement {
    Select {
        columns: Vec<Expression>,
        from: String,
        r#where: Option<Expression>,
        orderby: Vec<(Expression, Option<Order>)>, // Expression to order by, and ASC/DESC
    },
    CreateTable {
        table_name: String,
        column_list: Vec<TableColumn>,
    },
}

#[derive(Debug)]
pub enum Order {
    Asc,
    Desc,
}

pub fn parse(tokens: &[Token]) -> Result<Statement, String> {
    let mut iter = tokens.iter().enumerate().peekable();

    match iter.next() {
        Some((_, Keyword(k))) if k == "SELECT" => parse_select_statement(&mut iter, tokens),
        Some((_, Keyword(k))) if k == "CREATE" => parse_create_table_statement(&mut iter, tokens),
        _ => Err("Unsupported or invalid SQL statement".into()),
    }
}

fn parse_select_statement<'a, I>(iter: &mut std::iter::Peekable<I>, all_tokens: &'a [Token]) -> Result<Statement, String>
where
    I: Iterator<Item = (usize, &'a Token)>,
{
    let mut columns = vec![];
    loop {
        match iter.next() {
            Some((_, Identifier(name))) => columns.push(Expression::Identifier(name.to_string())),
            Some((_, StringLiteral(s))) => columns.push(Expression::String(s.to_string())),
            Some((_, Number(n))) => columns.push(Expression::Number(*n)),
            Some((_, BoolLiteral(b))) => columns.push(Expression::Bool(*b)),
            Some((_, Star)) => columns.push(Expression::Identifier("*".to_string())),
            Some((_, LParen)) => {
                let mut paren_level = 1;
                let mut inner_tokens = vec![LParen];
                let mut inner_count = 0;
                while let Some((_, token)) = iter.next() {
                    inner_tokens.push(token.clone());
                    inner_count += 1;
                    match token {
                        LParen => paren_level += 1,
                        RParen => paren_level -= 1,
                        EOF => return Err("Unclosed parenthesis".into()),
                        _ => {}
                    }
                    if paren_level == 0 {
                        break;
                    }
                }
                let (expr, _) = crate::expression::parse_expression(&inner_tokens[1..inner_tokens.len() - 1], 0)
                    .map_err(|e| format!("Error parsing expression in parentheses: {}", e))?;
                columns.push(expr);
            }
            Some((_, Comma)) => continue,
            Some((_, Keyword(k))) if k == "FROM" => break,
            Some((_, token)) => return Err(format!("Unexpected token in SELECT columns: {:?}", token)),
            None => return Err("Expected FROM clause".into()),
        }
    }

    let from = match iter.next() {
        Some((_, Identifier(name))) => name.to_string(),
        Some((_, token)) => return Err(format!("Expected table name after FROM, got: {:?}", token)),
        None => return Err("Expected table name after FROM".into()),
    };

    let mut r#where = None;
    if let Some((_, Keyword(k))) = iter.peek() {
        if k == "WHERE" {
            iter.next(); // Consume WHERE
            let start_index = iter.peek().map_or(all_tokens.len(), |(idx, _)| *idx);
            let remaining_slice = &all_tokens[start_index..];
            let (expr, consumed) = crate::expression::parse_expression(remaining_slice, 0)
                .map_err(|e| format!("Error parsing WHERE clause: {}", e))?;
            r#where = Some(expr);
            for _ in 0..consumed {
                if iter.next().is_none() {
                    break;
                }
            }
        }
    }

    let mut orderby = vec![];
    if let Some((_, Keyword(k))) = iter.peek() {
        if k == "ORDER" {
            iter.next(); // Consume ORDER
            if let Some((_, Keyword(by_k))) = iter.next() {
                if by_k == "BY" {
                    loop {
                        let start_index = iter.peek().map_or(all_tokens.len(), |(idx, _)| *idx);
                        let remaining_slice = &all_tokens[start_index..];
                        let (expr, consumed) = crate::expression::parse_expression(remaining_slice, 0)
                            .map_err(|e| format!("Error parsing ORDER BY expression: {}", e))?;
                        let order = match iter.peek() {
                            Some((_, Asc)) => {
                                iter.next();
                                Some(Order::Asc)
                            }
                            Some((_, Desc)) => {
                                iter.next();
                                Some(Order::Desc)
                            }
                            _ => None,
                        };
                        orderby.push((expr, order));
                        for _ in 0..consumed {
                            if iter.next().is_none() {
                                break;
                            }
                        }
                        if let Some((_, Comma)) = iter.peek() {
                            iter.next(); // Consume comma
                        } else {
                            break;
                        }
                    }
                } else {
                    return Err("Expected BY after ORDER".into());
                }
            } else {
                return Err("Expected BY after ORDER".into());
            }
        }
    }

    Ok(Statement::Select {
        columns,
        from,
        r#where,
        orderby,
    })
}

fn parse_create_table_statement<'a, I>(iter: &mut std::iter::Peekable<I>, all_tokens: &'a [Token]) -> Result<Statement, String>
where
    I: Iterator<Item = (usize, &'a Token)>,
{
    if let Some((_, Keyword(k))) = iter.next() {
        if k == "TABLE" {
            if let Some((_, Identifier(name))) = iter.next() {
                if let Some((_, LParen)) = iter.next() {
                    let mut column_list = Vec::new();
                    loop {
                        if let Some((_, RParen)) = iter.peek() {
                            iter.next();
                            break;
                        }
                        if let Some((_, Identifier(col_name))) = iter.next() {
                            let column = parse_table_column(col_name.to_string(), iter, all_tokens)?;
                            column_list.push(column);
                            if let Some((_, Comma)) = iter.peek() {
                                iter.next();
                            } else if let Some((_, RParen)) = iter.peek() {
                                continue;
                            } else {
                                return Err("Expected comma or closing parenthesis after column definition".into());
                            }
                        } else {
                            return Err("Expected column name".into());
                        }
                    }
                    return Ok(Statement::CreateTable {
                        table_name: name.to_string(),
                        column_list,
                    });
                } else {
                    return Err("Expected opening parenthesis after table name".into());
                }
            } else {
                return Err("Expected table name after CREATE TABLE".into());
            }
        } else {
            return Err("Expected TABLE after CREATE".into());
        }
    } else {
        return Err("Expected TABLE keyword".into());
    }
}

fn parse_table_column<'a, I>(
    column_name: String,
    iter: &mut std::iter::Peekable<I>,
    all_tokens: &'a [Token],
) -> Result<TableColumn, String>
where
    I: Iterator<Item = (usize, &'a Token)>,
{
    let column_type = match iter.next() {
        Some((_, Int)) => DBType::Int,
        Some((_, Varchar(len))) => DBType::Varchar(*len),
        Some((_, Bool)) => DBType::Bool,
        Some((_, token)) => return Err(format!("Unexpected data type: {:?}", token)),
        None => return Err("Expected data type".into()),
    };

    let mut constraints = Vec::new();
    while let Some((_, token)) = iter.peek() {
        match token {
            PrimaryKey => {
                constraints.push(Constraint::PrimaryKey);
                iter.next();
                if let Some((_, Keyword(k))) = iter.peek() {
                    if k == "KEY" {
                        iter.next();
                    }
                }
            }
            Keyword(k) if k == "NOT" => {
                iter.next();
                if let Some((_, Keyword(null_k))) = iter.next() {
                    if null_k == "NULL" {
                        constraints.push(Constraint::NotNull);
                    } else {
                        return Err("Expected NULL after NOT".into());
                    }
                } else {
                    return Err("Expected NULL after NOT".into());
                }
            }
            Check => {
                iter.next();
                if let Some((_, LParen)) = iter.next() {
                    let start_index = iter.peek().map_or(all_tokens.len(), |(idx, _)| *idx);
                    let remaining_slice = &all_tokens[start_index..];
                    let (expr, consumed) = crate::expression::parse_expression(remaining_slice, 0)
                        .map_err(|e| format!("Error parsing CHECK expression: {}", e))?;
                    constraints.push(Constraint::Check(expr));
                    for _ in 0..consumed {
                        if iter.next().is_none() {
                            break;
                        }
                    }
                    if let Some((_, RParen)) = iter.next() {
                    } else {
                        return Err("Expected closing parenthesis after CHECK expression".into());
                    }
                } else {
                    return Err("Expected opening parenthesis after CHECK".into());
                }
            }
            Comma | RParen => break,
            _ => break, // Stop if it's not a constraint keyword
        }
    }

    Ok(TableColumn {
        column_name,
        column_type,
        constraints,
    })
}

// Helper function to convert the rest of the iterator into a Vec of tokens
fn tokens_from_iterator<'a, I>(iter: &mut std::iter::Peekable<I>) -> Vec<&'a Token>
where
    I: Iterator<Item = &'a Token>,
{
    let mut tokens = Vec::new();
    while let Some(token) = iter.peek() {
        tokens.push(*token); // Dereference token
        iter.next();
    }
    tokens
}