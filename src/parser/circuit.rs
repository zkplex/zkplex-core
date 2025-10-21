//! Parser for ZKP circuits
//!
//! This module uses Pest to parse circuit strings into AST.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use super::ast::*;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "circuit.pest"]
struct CircuitParser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Pest parsing error: {0}")]
    PestError(#[from] pest::error::Error<Rule>),

    #[error("Invalid expression structure")]
    InvalidStructure,

    #[error("Unknown operator: {0}")]
    UnknownOperator(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Parse a circuit string into an Expression AST
pub fn parse_circuit(input: &str) -> ParseResult<Expression> {
    let pairs = CircuitParser::parse(Rule::circuit, input)?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::circuit => {
                // Get the expression inside
                if let Some(expr_pair) = pair.into_inner().next() {
                    return parse_expression(expr_pair);
                }
            }
            _ => {}
        }
    }

    Err(ParseError::InvalidStructure)
}

fn parse_expression(pair: Pair<Rule>) -> ParseResult<Expression> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair.into_inner().next().ok_or(ParseError::InvalidStructure)?;
            parse_boolean_or(inner)
        }
        _ => Err(ParseError::InvalidStructure),
    }
}

fn parse_boolean_or(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_boolean_and(inner.next().ok_or(ParseError::InvalidStructure)?)?;

    while let Some(op_or_right) = inner.next() {
        match op_or_right.as_rule() {
            Rule::or_op => {
                let right = parse_boolean_and(inner.next().ok_or(ParseError::InvalidStructure)?)?;
                left = Expression::or(left, right);
            }
            _ => {
                // If it's not an operator, it must be the right side of a previous operation
                left = Expression::or(left, parse_boolean_and(op_or_right)?);
            }
        }
    }

    Ok(left)
}

fn parse_boolean_and(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_comparison(inner.next().ok_or(ParseError::InvalidStructure)?)?;

    while let Some(op_or_right) = inner.next() {
        match op_or_right.as_rule() {
            Rule::and_op => {
                let right = parse_comparison(inner.next().ok_or(ParseError::InvalidStructure)?)?;
                left = Expression::and(left, right);
            }
            _ => {
                left = Expression::and(left, parse_comparison(op_or_right)?);
            }
        }
    }

    Ok(left)
}

fn parse_comparison(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let left = parse_additive(inner.next().ok_or(ParseError::InvalidStructure)?)?;

    if let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::comparison_op {
            let op = match op_pair.as_str() {
                ">" => ComparisonOperator::Greater,
                "<" => ComparisonOperator::Less,
                "==" => ComparisonOperator::Equal,
                ">=" => ComparisonOperator::GreaterEqual,
                "<=" => ComparisonOperator::LessEqual,
                "!=" => ComparisonOperator::NotEqual,
                _ => return Err(ParseError::UnknownOperator(op_pair.as_str().to_string())),
            };

            let right = parse_additive(inner.next().ok_or(ParseError::InvalidStructure)?)?;
            return Ok(Expression::compare(op, left, right));
        }
    }

    Ok(left)
}

fn parse_additive(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_multiplicative(inner.next().ok_or(ParseError::InvalidStructure)?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_rule() {
            Rule::add_op => BinaryOperator::Add,
            Rule::sub_op => BinaryOperator::Sub,
            _ => return Err(ParseError::InvalidStructure),
        };

        let right = parse_multiplicative(inner.next().ok_or(ParseError::InvalidStructure)?)?;
        left = Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_multiplicative(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_unary(inner.next().ok_or(ParseError::InvalidStructure)?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_rule() {
            Rule::mul_op => BinaryOperator::Mul,
            Rule::div_op => BinaryOperator::Div,
            _ => return Err(ParseError::InvalidStructure),
        };

        let right = parse_unary(inner.next().ok_or(ParseError::InvalidStructure)?)?;
        left = Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_unary(pair: Pair<Rule>) -> ParseResult<Expression> {
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or(ParseError::InvalidStructure)?;

    match first.as_rule() {
        Rule::not_op => {
            let operand = parse_unary(inner.next().ok_or(ParseError::InvalidStructure)?)?;
            Ok(Expression::not(operand))
        }
        Rule::neg_op => {
            let operand = parse_unary(inner.next().ok_or(ParseError::InvalidStructure)?)?;
            Ok(Expression::UnaryOp {
                op: UnaryOperator::Neg,
                operand: Box::new(operand),
            })
        }
        Rule::primary => parse_primary(first),
        _ => Err(ParseError::InvalidStructure),
    }
}

fn parse_primary(pair: Pair<Rule>) -> ParseResult<Expression> {
    let inner = pair.into_inner().next().ok_or(ParseError::InvalidStructure)?;

    match inner.as_rule() {
        Rule::number => Ok(Expression::Constant(inner.as_str().to_string())),
        Rule::variable => Ok(Expression::Variable(inner.as_str().to_string())),
        Rule::boolean => {
            let value = matches!(inner.as_str(), "true" | "TRUE");
            Ok(Expression::Boolean(value))
        }
        Rule::expression => parse_expression(inner),
        _ => Err(ParseError::InvalidStructure),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_arithmetic() {
        let expr = parse_circuit("A + B").unwrap();
        assert_eq!(expr.variables(), vec!["A", "B"]);
    }

    #[test]
    fn test_parse_complex_arithmetic() {
        let expr = parse_circuit("(A + B) * C").unwrap();
        assert_eq!(expr.variables(), vec!["A", "B", "C"]);
    }

    #[test]
    fn test_parse_comparison() {
        let expr = parse_circuit("A > B").unwrap();
        match expr {
            Expression::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOperator::Greater);
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_complex_comparison() {
        let expr = parse_circuit("(A + B) * C > D").unwrap();
        assert_eq!(expr.variables(), vec!["A", "B", "C", "D"]);
        match expr {
            Expression::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOperator::Greater);
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_boolean() {
        let expr = parse_circuit("A > B AND C < D").unwrap();
        match expr {
            Expression::BooleanOp { op, .. } => {
                assert_eq!(op, BooleanOperator::And);
            }
            _ => panic!("Expected boolean operation"),
        }
    }

    #[test]
    fn test_parse_not() {
        let expr = parse_circuit("NOT (A > B)").unwrap();
        match expr {
            Expression::UnaryOp { op: UnaryOperator::Not, .. } => {}
            _ => panic!("Expected NOT operation"),
        }
    }

    #[test]
    fn test_parse_precedence() {
        // Test that * has higher precedence than +
        let expr = parse_circuit("A + B * C").unwrap();
        match expr {
            Expression::BinaryOp {
                op: BinaryOperator::Add,
                right,
                ..
            } => {
                match *right {
                    Expression::BinaryOp {
                        op: BinaryOperator::Mul,
                        ..
                    } => {}
                    _ => panic!("Expected multiplication on right"),
                }
            }
            _ => panic!("Expected addition at top level"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse_circuit("(A + B) * C").unwrap();
        match expr {
            Expression::BinaryOp {
                op: BinaryOperator::Mul,
                left,
                ..
            } => {
                match *left {
                    Expression::BinaryOp {
                        op: BinaryOperator::Add,
                        ..
                    } => {}
                    _ => panic!("Expected addition on left"),
                }
            }
            _ => panic!("Expected multiplication at top level"),
        }
    }
}