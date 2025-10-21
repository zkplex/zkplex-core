//! Strategy validation module
//!
//! This module contains logic for validating that a chosen proof strategy
//! is compatible with the operations used in a circuit.

use crate::circuit::Circuit;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Proof generation strategy
///
/// Strategies control how circuit constraints are implemented, balancing between
/// proof size, proving time, and circuit size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Strategy {
    /// Adaptive strategy that automatically selects optimal strategy based on circuit
    Auto,
    /// Base strategy: arithmetic (+, -, *, /), equality (==, !=), and boolean (AND, OR, NOT) operations only
    Boolean,
    /// Full comparison support using lookup tables for fast proving
    Lookup,
    /// Full comparison support using bit decomposition approach
    #[serde(rename = "bitd")]
    BitD,
}

impl Strategy {
    /// Returns the string representation of the strategy
    pub fn as_str(&self) -> &'static str {
        match self {
            Strategy::Auto => "auto",
            Strategy::Boolean => "boolean",
            Strategy::Lookup => "lookup",
            Strategy::BitD => "bitd",
        }
    }

    /// Returns the supported operations for this strategy
    pub fn operations(&self) -> &'static str {
        match self {
            Strategy::Auto => "All operations (adaptive selection)",
            Strategy::Boolean => "+, -, *, /, ==, !=, AND, OR, NOT",
            Strategy::Lookup => "+, -, *, /, ==, !=, AND, OR, NOT, >, <, >=, <=",
            Strategy::BitD => "+, -, *, /, ==, !=, AND, OR, NOT, >, <, >=, <=",
        }
    }

    /// Returns a short description of the strategy
    pub fn description(&self) -> &'static str {
        match self {
            Strategy::Auto => "Adaptive strategy (automatically chooses optimal based on circuit)",
            Strategy::Boolean => "Base strategy (arithmetic, equality, and boolean operations)",
            Strategy::Lookup => "Full comparison support with lookup tables",
            Strategy::BitD => "Full comparison support with bit decomposition",
        }
    }

    /// Returns the use case / when to use this strategy
    pub fn use_case(&self) -> &'static str {
        match self {
            Strategy::Auto => "Default choice - automatically selects optimal strategy",
            Strategy::Boolean => "Circuits without range comparisons - smallest proofs",
            Strategy::Lookup => "Fast proving with comparisons (efficient for ≤16-bit values)",
            Strategy::BitD => "Comparisons with larger values (more efficient for >16-bit values)",
        }
    }

    /// Returns detailed information about the strategy
    pub fn detailed_info(&self) -> String {
        format!(
            "{}\nOperations: {}\nUse case: {}",
            self.description(),
            self.operations(),
            self.use_case()
        )
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Strategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Strategy::Auto),
            "boolean" => Ok(Strategy::Boolean),
            "lookup" => Ok(Strategy::Lookup),
            "bitd" => Ok(Strategy::BitD),
            _ => Err(format!(
                "Invalid strategy '{}'. Valid strategies: auto, boolean, lookup, bitd",
                s
            )),
        }
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Auto
    }
}

/// Validate that the chosen proof strategy is compatible with circuit operations
///
/// # Arguments
///
/// * `circuit` - The circuit to validate
/// * `strategy` - The proof strategy
///
/// # Returns
///
/// `Ok(())` if the strategy is compatible, or `Err(String)` with a detailed error message
///
/// # Example
///
/// ```ignore
/// let circuit = Circuit::from_program(&program)?;
/// validate_strategy_compatibility(&circuit, Strategy::Boolean)?;
/// ```
pub fn validate_strategy_compatibility(
    circuit: &Circuit,
    strategy: Strategy,
) -> Result<(), String> {
    match strategy {
        Strategy::Boolean => {
            // Boolean strategy is the base strategy supporting:
            // - Arithmetic: +, -, *, /
            // - Equality: ==, != (including implicit constrain_instance)
            // - Boolean: AND, OR, NOT
            // BUT NOT range comparisons (>, <, >=, <=)
            if circuit.uses_range_check_comparisons() {
                return Err(format!(
                    "Strategy '{}' does not support range comparison operations (>, <, >=, <=).\n\
                     \n\
                     The '{}' strategy only supports: {}\n\
                     \n\
                     Suggestions:\n\
                     - Use '{}' strategy: {}\n\
                     - Use '{}' strategy: {}\n\
                     - Use '{}' strategy: {}",
                    strategy.as_str(),
                    strategy.as_str(),
                    strategy.operations(),
                    Strategy::Lookup.as_str(),
                    Strategy::Lookup.description(),
                    Strategy::BitD.as_str(),
                    Strategy::BitD.description(),
                    Strategy::Auto.as_str(),
                    Strategy::Auto.description()
                ));
            }
        }
        Strategy::Lookup | Strategy::BitD | Strategy::Auto => {
            // These strategies support all operations
        }
    }

    Ok(())
}