use crate::tokenizer::Token;

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i64),
    Identifier(String),
    String(String),
    Bool(bool),
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Unknown(String), // fallback
}

pub fn parse_expression(tokens: &[Token], min_prec: u8) -> Result<(Expression, usize), String> {
    let mut pos = 0;

    let mut lhs = match tokens.get(pos) {
        Some(Token::Number(n)) => {
            pos += 1;
            Expression::Number(*n)
        }
        Some(Token::StringLiteral(s)) => {
            pos += 1;
            Expression::String(s.clone())
        }
        Some(Token::BoolLiteral(b)) => {
            pos += 1;
            Expression::Bool(*b)
        }
        Some(Token::Identifier(name)) => {
            pos += 1;
            Expression::Identifier(name.clone())
        }
        Some(Token::Keyword(k)) if k == "NOT" => {
            pos += 1;
            let (inner_expr, consumed) = parse_expression(&tokens[pos..], 6)?; // 6 = higher than any binary op
            pos += consumed;
            Expression::UnaryOp {
                op: UnaryOperator::Not,
                expr: Box::new(inner_expr),
            }
        }
        Some(Token::LParen) => {
            pos += 1;
            let (expr, consumed) = parse_expression(&tokens[pos..], 0)?;
            pos += consumed;
            match tokens.get(pos) {
                Some(Token::RParen) => {
                    pos += 1;
                    expr
                }
                _ => return Err("Expected ')'".to_string()),
            }
        }
        _ => return Err("Unexpected token at beginning of expression".to_string()),
    };

    loop {
        let op_token = match tokens.get(pos) {
            Some(Token::Operator(op)) => op.clone(),
            _ => break,
        };

        let prec = get_precedence(&op_token);
        if prec < min_prec {
            break;
        }

        let binary_op = match to_binary_operator(&op_token) {
            Some(op) => op,
            None => return Err(format!("Unknown operator '{}'", op_token)),
        };

        pos += 1;
        let (rhs, consumed) = parse_expression(&tokens[pos..], prec + 1)?;
        pos += consumed;

        lhs = Expression::BinaryOp {
            left: Box::new(lhs),
            op: binary_op,
            right: Box::new(rhs),
        };
    }

    Ok((lhs, pos))
}

fn get_precedence(op: &str) -> u8 {
    match op {
        "OR" => 1,
        "AND" => 2,
        "=" | "<" | ">" | "<=" | ">=" => 3,
        "+" | "-" => 4,
        "*" | "/" => 5,
        _ => 0,
    }
}

fn to_binary_operator(op: &str) -> Option<BinaryOperator> {
    match op {
        "OR" => Some(BinaryOperator::Or),
        "AND" => Some(BinaryOperator::And),
        "=" => Some(BinaryOperator::Equal),
        "!=" => Some(BinaryOperator::NotEqual),
        "<" => Some(BinaryOperator::Less),
        "<=" => Some(BinaryOperator::LessEqual),
        ">" => Some(BinaryOperator::Greater),
        ">=" => Some(BinaryOperator::GreaterEqual),
        "+" => Some(BinaryOperator::Add),
        "-" => Some(BinaryOperator::Subtract),
        "*" => Some(BinaryOperator::Multiply),
        "/" => Some(BinaryOperator::Divide),
        other => Some(BinaryOperator::Unknown(other.to_string())),
    }
}
