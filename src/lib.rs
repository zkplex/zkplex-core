//! ZKPlex WASM Library
//!
//! This library provides WASM bindings for creating and verifying
//! zero-knowledge proofs based on circuits.
//!
//! # Example Circuits
//!
//! Simple arithmetic and comparison:
//! ```ignore
//! (A + B) * C > D
//! ```
//!
//! With boolean conditions:
//! ```ignore
//! (A + B) > C AND D < E
//! (A > B) OR (C == D)
//! NOT (A >= B) AND (C + D) > E
//! ```
//!
//! Complex nested conditions:
//! ```ignore
//! ((A + B) * C > D) AND ((E - F) <= G) OR (H != I)
//! ```
//!
//! # Supported Value Formats
//!
//! - **Decimal**: `"12345"`
//! - **Hexadecimal**: `"0x1a2b"` (Ethereum addresses, hashes)
//! - **Base58**: `"5HpH..."` (Solana/Bitcoin keys)
//! - **Base64**: `"SGVsbG8="` (Universal encoding)
//!
//! # Outputs are Numbers
//!
//! - Comparisons return: **0** (false) or **1** (true)
//! - Boolean operations return: **0** or **1**
//! - Arithmetic operations return: any number
//!
//! # Public vs Secret Signals (Witnesses)
//!
//! - **Secret signals** (`public: false`): Witnesses - secret values
//! - **Public signals** (`public: true`): Values known to verifier

// Core modules
pub mod api;
pub mod circuit;
pub mod encoding;
pub mod layout;
pub mod parser;
pub mod preprocess;
pub mod wasm;

// Re-export commonly used types
pub use api::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse, Signal};
pub use encoding::{ValueEncoding, parse_value, parse_value_auto};
pub use parser::{Expression, BinaryOperator, ComparisonOperator, BooleanOperator, UnaryOperator, parse_circuit, ParseError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arithmetic_comparison() {
        let circuit = "(A + B) * C > D";
        let expr = parse_circuit(circuit).unwrap();
        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_parse_boolean_and() {
        let circuit = "(A + B) > C AND D < E";
        let expr = parse_circuit(circuit).unwrap();
        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C", "D", "E"]);
    }

    #[test]
    fn test_parse_boolean_or() {
        let circuit = "(A > B) OR (C == D)";
        let expr = parse_circuit(circuit).unwrap();
        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_parse_boolean_not() {
        let circuit = "NOT (A >= B) AND (C + D) > E";
        let expr = parse_circuit(circuit).unwrap();
        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C", "D", "E"]);
    }

    #[test]
    fn test_parse_complex_nested() {
        let circuit = "((A + B) * C > D) AND ((E - F) <= G) OR (H != I)";
        let expr = parse_circuit(circuit).unwrap();
        let vars = expr.variables();
        assert_eq!(vars, vec!["A", "B", "C", "D", "E", "F", "G", "H", "I"]);
    }
}
