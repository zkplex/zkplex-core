//! Abstract Syntax Tree for ZKP circuits
//!
//! This module defines the AST structure for representing circuits
//! that can be compiled into Halo2 circuits.
//!
//! # Supported Operations
//!
//! ## Arithmetic Operations
//! - Addition: `+`
//! - Subtraction: `-`
//! - Multiplication: `*`
//! - Division: `/`
//!
//! ## Comparison Operations (return 0 or 1)
//! All comparisons return binary outputs:
//! - `3 < 5` → 1 (true)
//! - `3 >= 5` → 0 (false)
//!
//! Operators: `>`, `<`, `==`, `>=`, `<=`, `!=`
//!
//! ## Boolean Operations (treat any non-zero as true)
//!
//! ### AND
//! Returns 1 if both operands are non-zero, otherwise 0:
//! - `1 AND 1` → 1
//! - `1 AND 4` → 1
//! - `5 AND 2` → 1
//! - `5 AND 0` → 0
//!
//! ### OR
//! Returns 1 if at least one operand is non-zero, otherwise 0:
//! - `1 OR 0` → 1
//! - `5 OR 3` → 1
//! - `0 OR 0` → 0
//!
//! ### NOT
//! Returns 1 if operand is 0, otherwise 0:
//! - `NOT 0` → 1
//! - `NOT 1` → 0
//! - `NOT 5` → 0
//! - `NOT 123` → 0
//!
//! ## Precedence
//! Parentheses can be used to control operation order

use serde::{Deserialize, Serialize};

/// Expression in the circuit AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Variable (can be public or secret)
    Variable(String),

    /// Constant value (as string to support big numbers)
    Constant(String),

    /// Binary arithmetic operation
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    /// Comparison operation (produces boolean)
    Comparison {
        op: ComparisonOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Boolean operation
    BooleanOp {
        op: BooleanOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Boolean constant
    Boolean(bool),
}

/// Binary arithmetic operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Neg,      // -x (negation)
    Not,      // NOT x (boolean not)
}

/// Comparison operators (require range checks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Greater,        // >
    Less,           // <
    Equal,          // ==
    GreaterEqual,   // >=
    LessEqual,      // <=
    NotEqual,       // !=
}

/// Boolean operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOperator {
    And,    // AND
    Or,     // OR
}

impl Expression {
    /// Helper to create a variable expression
    pub fn var(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }

    /// Helper to create a constant expression
    pub fn constant(value: impl Into<String>) -> Self {
        Expression::Constant(value.into())
    }

    /// Helper to create an addition expression
    pub fn add(left: Expression, right: Expression) -> Self {
        Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a subtraction expression
    pub fn sub(left: Expression, right: Expression) -> Self {
        Expression::BinaryOp {
            op: BinaryOperator::Sub,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a multiplication expression
    pub fn mul(left: Expression, right: Expression) -> Self {
        Expression::BinaryOp {
            op: BinaryOperator::Mul,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a division expression
    pub fn div(left: Expression, right: Expression) -> Self {
        Expression::BinaryOp {
            op: BinaryOperator::Div,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a comparison expression
    pub fn compare(op: ComparisonOperator, left: Expression, right: Expression) -> Self {
        Expression::Comparison {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a boolean AND expression
    pub fn and(left: Expression, right: Expression) -> Self {
        Expression::BooleanOp {
            op: BooleanOperator::And,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a boolean OR expression
    pub fn or(left: Expression, right: Expression) -> Self {
        Expression::BooleanOp {
            op: BooleanOperator::Or,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a NOT expression
    pub fn not(operand: Expression) -> Self {
        Expression::UnaryOp {
            op: UnaryOperator::Not,
            operand: Box::new(operand),
        }
    }

    /// Get all variable names used in this expression
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Expression::Variable(name) => vars.push(name.clone()),
            Expression::Constant(_) | Expression::Boolean(_) => {}
            Expression::BinaryOp { left, right, .. } => {
                left.collect_variables(vars);
                right.collect_variables(vars);
            }
            Expression::UnaryOp { operand, .. } => {
                operand.collect_variables(vars);
            }
            Expression::Comparison { left, right, .. } => {
                left.collect_variables(vars);
                right.collect_variables(vars);
            }
            Expression::BooleanOp { left, right, .. } => {
                left.collect_variables(vars);
                right.collect_variables(vars);
            }
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Variable(name) => write!(f, "{}", name),
            Expression::Constant(value) => write!(f, "{}", value),
            Expression::Boolean(b) => write!(f, "{}", b),
            Expression::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expression::UnaryOp { op, operand } => {
                write!(f, "({}{})", op, operand)
            }
            Expression::Comparison { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expression::BooleanOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
        }
    }
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
        }
    }
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "-"),
            UnaryOperator::Not => write!(f, "NOT "),
        }
    }
}

impl std::fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Greater => write!(f, ">"),
            ComparisonOperator::Less => write!(f, "<"),
            ComparisonOperator::Equal => write!(f, "=="),
            ComparisonOperator::GreaterEqual => write!(f, ">="),
            ComparisonOperator::LessEqual => write!(f, "<="),
            ComparisonOperator::NotEqual => write!(f, "!="),
        }
    }
}

impl std::fmt::Display for BooleanOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BooleanOperator::And => write!(f, "AND"),
            BooleanOperator::Or => write!(f, "OR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        // (A + B) * C
        let expr = Expression::mul(
            Expression::add(
                Expression::var("A"),
                Expression::var("B"),
            ),
            Expression::var("C"),
        );

        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_comparison_expression() {
        // (A + B) > D
        let expr = Expression::compare(
            ComparisonOperator::Greater,
            Expression::add(
                Expression::var("A"),
                Expression::var("B"),
            ),
            Expression::var("D"),
        );

        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "D"]);
    }

    #[test]
    fn test_boolean_expression() {
        // (A > B) AND (C < D)
        let expr = Expression::and(
            Expression::compare(
                ComparisonOperator::Greater,
                Expression::var("A"),
                Expression::var("B"),
            ),
            Expression::compare(
                ComparisonOperator::Less,
                Expression::var("C"),
                Expression::var("D"),
            ),
        );

        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_display() {
        let expr = Expression::mul(
            Expression::add(
                Expression::var("A"),
                Expression::var("B"),
            ),
            Expression::var("C"),
        );

        assert_eq!(expr.to_string(), "((A + B) * C)");
    }
}