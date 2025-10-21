//! Circuit builder for converting AST to Halo2 circuits
//!
//! This module provides functionality to convert parsed circuit AST
//! into Halo2 circuits that can be proven and verified.

use crate::parser::ast::*;
use crate::encoding::{parse_value, parse_value_auto};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    pasta::Fp,
    plonk::{
        Advice, Circuit as PlonkCircuit, Column, ConstraintSystem, Error,
        Instance, Selector,
    },
};
use halo2_proofs::plonk::gadgets::{
    comparison::{ComparisonConfig, ComparisonChip},
};
use std::collections::HashMap;
use ff::{Field, PrimeField};
use num_bigint::BigUint;
use num_traits::Num;

/// Configuration for the circuit
#[derive(Debug, Clone)]
pub struct CircuitConfig {
    /// Advice columns for computation
    pub advice: Vec<Column<Advice>>,

    /// Instance column for public inputs/outputs
    pub instance: Column<Instance>,

    /// Selectors for operations
    pub s_add: Selector,
    pub s_mul: Selector,

    /// Comparison gadget configuration (None for arithmetic-only circuits)
    /// When None, circuit can only do arithmetic (+, -, *, /) - saves 7 columns!
    pub comparison: Option<ComparisonConfig>,
}

impl CircuitConfig {
    /// Configure the circuit with necessary columns and gates
    ///
    /// Uses default threshold (balanced strategy) with comparison support.
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        Self::configure_with_strategy(meta, 16) // Default: balanced (auto)
    }

    /// Configure boolean circuit with comparison support for boolean operations
    ///
    /// Creates columns needed for arithmetic and boolean operations (AND, OR, NOT).
    /// Uses only is_zero gadget without range checks, keeping proofs smaller than full comparison.
    ///
    /// Use this for circuits that need boolean operations but not ordering comparisons.
    pub fn configure_boolean(meta: &mut ConstraintSystem<Fp>) -> Self {
        use halo2_proofs::plonk::gadgets::range_check_manager::RangeCheckManager;

        // Create advice columns for intermediate values
        // Only 3 columns needed for arithmetic: [a, b, output]
        let advice = vec![
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        // Enable equality constraints
        for col in &advice {
            meta.enable_equality(*col);
        }

        // Instance column for public inputs/outputs
        let instance = meta.instance_column();
        meta.enable_equality(instance);

        // Selectors for operations
        let s_add = meta.selector();
        let s_mul = meta.selector();

        // Configure minimal comparison support for boolean operations
        // We only need is_zero gadget for boolean ops, no range checks
        // This uses minimal threshold (0) to avoid lookup tables
        let range_check = RangeCheckManager::configure_with_threshold(meta, 0);
        let comparison = ComparisonConfig::configure(meta, range_check);

        // Define custom gates for addition and multiplication
        meta.create_gate("add_gate", |meta| {
            let s = meta.query_selector(s_add);
            let a = meta.query_advice(advice[0], halo2_proofs::poly::Rotation::cur());
            let b = meta.query_advice(advice[1], halo2_proofs::poly::Rotation::cur());
            let c = meta.query_advice(advice[2], halo2_proofs::poly::Rotation::cur());

            // Enforce: a + b = c
            vec![s * (a + b - c)]
        });

        meta.create_gate("mul_gate", |meta| {
            let s = meta.query_selector(s_mul);
            let a = meta.query_advice(advice[0], halo2_proofs::poly::Rotation::cur());
            let b = meta.query_advice(advice[1], halo2_proofs::poly::Rotation::cur());
            let c = meta.query_advice(advice[2], halo2_proofs::poly::Rotation::cur());

            // Enforce: a * b = c
            vec![s * (a * b - c)]
        });

        Self {
            advice,
            instance,
            s_add,
            s_mul,
            comparison: Some(comparison), // Minimal comparison support for boolean ops
        }
    }

    /// Configure with custom range check strategy
    ///
    /// # Arguments
    ///
    /// * `meta` - The constraint system
    /// * `threshold` - Threshold for lookup vs bit decomposition:
    ///   - 0 = always use bit decomposition (smallest proofs)
    ///   - 16 = balanced (default)
    ///   - 20 = prefer lookup tables (fastest proving)
    pub fn configure_with_strategy(meta: &mut ConstraintSystem<Fp>, threshold: usize) -> Self {
        use halo2_proofs::plonk::gadgets::range_check_manager::RangeCheckManager;

        // Create advice columns for intermediate values
        // Reduced from 4 to 3 to minimize proof size (each column adds ~3-4 KB overhead)
        // 3 columns is the minimum needed for our gates: [a, b, output]
        let advice = vec![
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        // Enable equality constraints
        for col in &advice {
            meta.enable_equality(*col);
        }

        // Instance column for public inputs/outputs
        let instance = meta.instance_column();
        meta.enable_equality(instance);

        // Selectors for operations
        let s_add = meta.selector();
        let s_mul = meta.selector();

        // Configure range check manager with custom threshold
        let range_check = RangeCheckManager::configure_with_threshold(meta, threshold);

        // Configure comparison gadget with range check
        let comparison = ComparisonConfig::configure(meta, range_check);

        // Define custom gates for addition and multiplication
        meta.create_gate("add_gate", |meta| {
            let s = meta.query_selector(s_add);
            let a = meta.query_advice(advice[0], halo2_proofs::poly::Rotation::cur());
            let b = meta.query_advice(advice[1], halo2_proofs::poly::Rotation::cur());
            let c = meta.query_advice(advice[2], halo2_proofs::poly::Rotation::cur());

            // Enforce: a + b = c
            vec![s * (a + b - c)]
        });

        meta.create_gate("mul_gate", |meta| {
            let s = meta.query_selector(s_mul);
            let a = meta.query_advice(advice[0], halo2_proofs::poly::Rotation::cur());
            let b = meta.query_advice(advice[1], halo2_proofs::poly::Rotation::cur());
            let c = meta.query_advice(advice[2], halo2_proofs::poly::Rotation::cur());

            // Enforce: a * b = c
            vec![s * (a * b - c)]
        });

        Self {
            advice,
            instance,
            s_add,
            s_mul,
            comparison: Some(comparison), // Wrap in Some
        }
    }
}

/// Statement in a circuit
#[derive(Debug, Clone)]
pub enum Statement {
    /// Assignment: variable <== expression
    Assignment { name: String, expression: Expression },
    /// Expression (used for final output)
    Expression(Expression),
}

/// Circuit for proving circuits
///
/// # Example
///
/// ```ignore
/// // Circuit: (A + B) > C
/// let mut signals = HashMap::new();
/// signals.insert("A", Fp::from(10));  // secret (witness)
/// signals.insert("B", Fp::from(20));  // secret (witness)
/// signals.insert("C", Fp::from(25));  // public
///
/// let circuit = Circuit::new(
///     expression,
///     signals,
///     vec!["C".to_string()],  // only C is public
/// );
/// ```
#[derive(Clone)]
pub struct Circuit {
    /// The circuit expression (AST) - kept for backwards compatibility
    pub expression: Option<Expression>,

    /// Circuit statements (for multi-statement circuits with intermediate signals)
    pub statements: Vec<Statement>,

    /// All signal values (variable name -> field element value)
    /// Contains BOTH public and secret (witness) signals
    pub signals: HashMap<String, Fp>,

    /// Names of public signals (subset of signals.keys())
    /// Secret signals = signals.keys() - public_signal_names
    pub public_signal_names: Vec<String>,

    /// Circuit output value (result of evaluating the main expression/last statement)
    /// This is constrained as an additional public signal during proof generation
    /// and verified during proof verification.
    /// NOTE: This is separate from user-defined signals, so users can have their own "output" signal
    pub circuit_output: Option<Fp>,

    /// Maximum bit size required for range checks (cached value)
    /// This is preserved even in without_witnesses() to ensure consistent lookup table loading
    pub cached_max_bits: Option<usize>,

    /// Range check strategy: "auto", "lookup", or "bitd"
    /// - "auto": Choose based on max_bits (balanced)
    /// - "lookup": Always use lookup tables (faster proving)
    /// - "bitd": Always use bit decomposition (smaller proofs)
    pub strategy: String,
}

impl Default for Circuit {
    fn default() -> Self {
        Self {
            expression: None,
            statements: Vec::new(),
            signals: HashMap::new(),
            public_signal_names: Vec::new(),
            circuit_output: None,
            cached_max_bits: None,
            strategy: "auto".to_string(),
        }
    }
}

impl Circuit {
    /// Create a new circuit
    pub fn new(
        expression: Expression,
        signals: HashMap<String, Fp>,
        public_signal_names: Vec<String>,
    ) -> Self {
        // Evaluate circuit output before moving signals
        let circuit_output = evaluate_expression(&expression, &signals).ok();

        let mut circuit = Self {
            expression: Some(expression),
            statements: Vec::new(),  // Empty for backwards compatibility
            signals,
            public_signal_names,
            circuit_output,
            cached_max_bits: None,
            strategy: "auto".to_string(),
        };

        // Compute and cache max_bits from signal values
        circuit.cached_max_bits = circuit.compute_max_range_check_bits();

        circuit
    }

    /// Check if circuit uses ordering comparisons that require range checks
    ///
    /// Range checks are required ONLY for ordering comparisons: >, <, >=, <=
    /// They are NOT required for:
    /// - Equality comparisons: ==, != (use is_zero gadget only)
    /// - Simple arithmetic: +, -, *, /
    /// - Boolean operations: AND, OR, NOT (use is_zero gadget)
    ///
    /// Returns true only if circuit uses >, <, >=, <=
    pub fn uses_range_check_comparisons(&self) -> bool {
        // Check main expression
        if let Some(expr) = &self.expression {
            if Self::expr_uses_ordering_comparisons(expr) {
                return true;
            }
        }

        // Check all statements
        for stmt in &self.statements {
            match stmt {
                Statement::Assignment { expression, .. } => {
                    if Self::expr_uses_ordering_comparisons(expression) {
                        return true;
                    }
                }
                Statement::Expression(expression) => {
                    if Self::expr_uses_ordering_comparisons(expression) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if circuit uses boolean operations (AND, OR, NOT)
    pub fn uses_boolean_operations(&self) -> bool {
        // Check main expression
        if let Some(expr) = &self.expression {
            if Self::expr_uses_boolean_ops(expr) {
                return true;
            }
        }

        // Check all statements
        for stmt in &self.statements {
            match stmt {
                Statement::Assignment { expression, .. } => {
                    if Self::expr_uses_boolean_ops(expression) {
                        return true;
                    }
                }
                Statement::Expression(expression) => {
                    if Self::expr_uses_boolean_ops(expression) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if circuit uses equality comparisons (==, !=)
    pub fn uses_equality_comparisons(&self) -> bool {
        // Check main expression
        if let Some(expr) = &self.expression {
            if Self::expr_uses_equality_comparisons(expr) {
                return true;
            }
        }

        // Check all statements
        for stmt in &self.statements {
            match stmt {
                Statement::Assignment { expression, .. } => {
                    if Self::expr_uses_equality_comparisons(expression) {
                        return true;
                    }
                }
                Statement::Expression(expression) => {
                    if Self::expr_uses_equality_comparisons(expression) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Recursively check if expression contains ordering comparisons (>, <, >=, <=)
    /// Returns false for ==, != as they don't need range checks
    fn expr_uses_ordering_comparisons(expr: &Expression) -> bool {
        use crate::parser::ComparisonOperator;

        match expr {
            Expression::Comparison { op, left, right } => {
                // Only ordering comparisons need range checks
                let needs_range_check = matches!(
                    op,
                    ComparisonOperator::Greater
                        | ComparisonOperator::Less
                        | ComparisonOperator::GreaterEqual
                        | ComparisonOperator::LessEqual
                );

                // Also check recursively in sub-expressions
                needs_range_check
                    || Self::expr_uses_ordering_comparisons(left)
                    || Self::expr_uses_ordering_comparisons(right)
            }

            Expression::BinaryOp { left, right, .. } => {
                Self::expr_uses_ordering_comparisons(left)
                    || Self::expr_uses_ordering_comparisons(right)
            }

            Expression::UnaryOp { operand, .. } => {
                Self::expr_uses_ordering_comparisons(operand)
            }

            Expression::BooleanOp { left, right, .. } => {
                Self::expr_uses_ordering_comparisons(left)
                    || Self::expr_uses_ordering_comparisons(right)
            }

            Expression::Variable(_) | Expression::Constant(_) | Expression::Boolean(_) => false,
        }
    }

    /// Recursively check if expression contains boolean operations (AND, OR, NOT)
    fn expr_uses_boolean_ops(expr: &Expression) -> bool {
        match expr {
            Expression::BooleanOp { left, right, .. } => {
                // Found a boolean op, also check recursively in sub-expressions
                true || Self::expr_uses_boolean_ops(left) || Self::expr_uses_boolean_ops(right)
            }
            Expression::UnaryOp { op: UnaryOperator::Not, operand } => {
                // Found NOT operation, also check recursively
                true || Self::expr_uses_boolean_ops(operand)
            }

            Expression::BinaryOp { left, right, .. } |
            Expression::Comparison { left, right, .. } => {
                Self::expr_uses_boolean_ops(left) || Self::expr_uses_boolean_ops(right)
            }

            Expression::UnaryOp { operand, .. } => Self::expr_uses_boolean_ops(operand),

            Expression::Variable(_) | Expression::Constant(_) | Expression::Boolean(_) => false,
        }
    }

    /// Recursively check if expression contains equality comparisons (==, !=)
    fn expr_uses_equality_comparisons(expr: &Expression) -> bool {
        use crate::parser::ComparisonOperator;

        match expr {
            Expression::Comparison { op, left, right } => {
                let is_equality = matches!(
                    op,
                    ComparisonOperator::Equal | ComparisonOperator::NotEqual
                );

                is_equality
                    || Self::expr_uses_equality_comparisons(left)
                    || Self::expr_uses_equality_comparisons(right)
            }

            Expression::BinaryOp { left, right, .. } |
            Expression::BooleanOp { left, right, .. } => {
                Self::expr_uses_equality_comparisons(left)
                    || Self::expr_uses_equality_comparisons(right)
            }

            Expression::UnaryOp { operand, .. } => {
                Self::expr_uses_equality_comparisons(operand)
            }

            Expression::Variable(_) | Expression::Constant(_) | Expression::Boolean(_) => false,
        }
    }

    /// Get maximum bit size needed for range checks in this circuit
    ///
    /// Returns the cached max_bits value if available (preserved from without_witnesses),
    /// otherwise computes it from signal values.
    ///
    /// Returns None if circuit doesn't use ordering comparisons (range checks not needed)
    pub fn max_range_check_bits(&self) -> Option<usize> {
        // Return cached value if available (set during construction or from without_witnesses)
        if let Some(cached) = self.cached_max_bits {
            return Some(cached);
        }

        // Otherwise compute from current signal values
        self.compute_max_range_check_bits()
    }

    /// Compute maximum bit size needed for range checks from signal values
    ///
    /// **OPTIMIZED**: Only analyzes values that are ACTUALLY used in ordering comparisons.
    ///
    /// For example, in `(key1 == key2) AND (age > 18)`:
    /// - `key1 == key2` returns 0 or 1 (no range check needed for ==)
    /// - `age > 18` compares `age` and `18` (range check needed)
    /// - Output: analyze only `age` and `18`, NOT key1/key2!
    ///
    /// This dramatically reduces k for circuits with equality checks on large values.
    ///
    /// Returns None if circuit doesn't use ordering comparisons (range checks not needed)
    fn compute_max_range_check_bits(&self) -> Option<usize> {
        // If no ordering comparisons, range checks not needed
        if !self.uses_range_check_comparisons() {
            return None;
        }

        // If signals are empty, we can't determine the size - return None
        // This will be handled by cached_max_bits in without_witnesses()
        if self.signals.is_empty() {
            return None;
        }

        // Find maximum value across values used in ordering comparisons
        let mut max_bits = 8; // Start with minimum

        // Analyze main expression
        if let Some(expr) = &self.expression {
            if let Some(bits) = self.max_bits_in_ordering_comparisons(expr) {
                if bits > max_bits {
                    max_bits = bits;
                }
            }
        }

        // Analyze all statements
        for stmt in &self.statements {
            let expr = match stmt {
                Statement::Assignment { expression, .. } => expression,
                Statement::Expression(expression) => expression,
            };

            if let Some(bits) = self.max_bits_in_ordering_comparisons(expr) {
                if bits > max_bits {
                    max_bits = bits;
                }
            }
        }

        Some(max_bits)
    }

    /// Recursively find maximum bit size of values used in ordering comparisons
    ///
    /// Returns None if no ordering comparisons found in this expression
    fn max_bits_in_ordering_comparisons(&self, expr: &Expression) -> Option<usize> {
        use crate::parser::ComparisonOperator;

        match expr {
            Expression::Comparison { op, left, right } => {
                // Check if this is an ordering comparison
                let is_ordering = matches!(
                    op,
                    ComparisonOperator::Greater
                        | ComparisonOperator::Less
                        | ComparisonOperator::GreaterEqual
                        | ComparisonOperator::LessEqual
                );

                if is_ordering {
                    // Evaluate left and right to get their actual values
                    let left_bits = self.evaluate_and_get_bits(left);
                    let right_bits = self.evaluate_and_get_bits(right);

                    // Return maximum of both sides
                    let mut max = left_bits.max(right_bits);

                    // Also check recursively in sub-expressions
                    if let Some(sub_bits) = self.max_bits_in_ordering_comparisons(left) {
                        max = max.max(sub_bits);
                    }
                    if let Some(sub_bits) = self.max_bits_in_ordering_comparisons(right) {
                        max = max.max(sub_bits);
                    }

                    Some(max)
                } else {
                    // ==, != don't need range checks, but check recursively
                    let left_bits = self.max_bits_in_ordering_comparisons(left);
                    let right_bits = self.max_bits_in_ordering_comparisons(right);

                    match (left_bits, right_bits) {
                        (Some(l), Some(r)) => Some(l.max(r)),
                        (Some(bits), None) | (None, Some(bits)) => Some(bits),
                        (None, None) => None,
                    }
                }
            }

            Expression::BinaryOp { left, right, .. } => {
                let left_bits = self.max_bits_in_ordering_comparisons(left);
                let right_bits = self.max_bits_in_ordering_comparisons(right);

                match (left_bits, right_bits) {
                    (Some(l), Some(r)) => Some(l.max(r)),
                    (Some(bits), None) | (None, Some(bits)) => Some(bits),
                    (None, None) => None,
                }
            }

            Expression::UnaryOp { operand, .. } => {
                self.max_bits_in_ordering_comparisons(operand)
            }

            Expression::BooleanOp { left, right, .. } => {
                let left_bits = self.max_bits_in_ordering_comparisons(left);
                let right_bits = self.max_bits_in_ordering_comparisons(right);

                match (left_bits, right_bits) {
                    (Some(l), Some(r)) => Some(l.max(r)),
                    (Some(bits), None) | (None, Some(bits)) => Some(bits),
                    (None, None) => None,
                }
            }

            Expression::Variable(_) | Expression::Constant(_) | Expression::Boolean(_) => None,
        }
    }

    /// Evaluate expression and get bit size of the output
    ///
    /// This evaluates the expression with current signal values to get the actual
    /// runtime value, which may be much smaller than the inputs.
    ///
    /// For example: `key1 == key2` where both are 256-bit returns 0 or 1 (8 bits)
    fn evaluate_and_get_bits(&self, expr: &Expression) -> usize {
        match evaluate_expression(expr, &self.signals) {
            Ok(value) => Self::field_to_bits(&value),
            Err(_) => {
                // If evaluation fails (e.g., variable not found), analyze structurally
                self.structural_max_bits(expr)
            }
        }
    }

    /// Structural analysis when evaluation is not possible
    ///
    /// Used as fallback when we don't have signal values (e.g., in without_witnesses)
    fn structural_max_bits(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Variable(name) => {
                // Get value if available
                self.signals.get(name)
                    .map(|v| Self::field_to_bits(v))
                    .unwrap_or(64) // Conservative: assume 64 bits if unknown
            }

            Expression::Constant(s) => {
                // Parse constant and get its bit size
                if let Ok(value) = parse_constant_to_field(s) {
                    Self::field_to_bits(&value)
                } else {
                    64 // Conservative fallback
                }
            }

            Expression::Boolean(_) => 8, // Booleans are 0 or 1 (8 bits minimum)

            Expression::Comparison { .. } => 8, // Comparisons return 0 or 1 (8 bits)

            Expression::BinaryOp { left, right, .. } => {
                // Arithmetic can increase bit size
                let left_bits = self.structural_max_bits(left);
                let right_bits = self.structural_max_bits(right);
                (left_bits + right_bits).min(64) // Cap at 64 bits
            }

            Expression::UnaryOp { operand, .. } => {
                self.structural_max_bits(operand)
            }

            Expression::BooleanOp { .. } => 8, // Boolean ops return 0 or 1 (8 bits)
        }
    }

    /// Determine minimum bit size needed for a field element
    fn field_to_bits(value: &Fp) -> usize {
        let bytes = value.to_repr();

        // Find the position of the highest non-zero byte
        let mut highest_byte_pos = None;
        for (i, byte) in bytes.as_ref().iter().enumerate().rev() {
            if *byte != 0 {
                highest_byte_pos = Some(i);
                break;
            }
        }

        let bits_needed = match highest_byte_pos {
            None => 0, // Value is zero
            Some(pos) => {
                let byte = bytes.as_ref()[pos];
                let bits_in_byte = 8 - byte.leading_zeros() as usize;
                pos * 8 + bits_in_byte
            }
        };

        // Round up to next supported size (8, 16, 32, or 64 bits)
        // Values requiring > 64 bits cannot use ordering comparisons
        match bits_needed {
            0 => 8,
            1..=8 => 8,
            9..=16 => 16,
            17..=32 => 32,
            _ => 64,  // 33+ bits → cap at 64 (max supported by range_check_manager)
        }
    }


    /// Build circuit from Zircon Program format
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Program: sum<==A+B;sum*2
    /// let program = Program::from_zircon("1/A:10,B:20/-/sum<==A+B;sum*2")?;
    /// let circuit = Circuit::from_program(&program)?;
    /// ```
    pub fn from_program(program: &crate::api::Program) -> Result<Self, String> {
        use crate::parser::parse_circuit;

        // Convert all input signals (secret + public) to field elements
        let mut signal_values = HashMap::new();
        let mut public_signal_names = Vec::new();

        // Process secret signals
        for (name, signal) in &program.secret {
            // Skip if value is None or "?" (placeholder)
            let value = match &signal.value {
                Some(v) => {
                    if v == "?" {
                        // "?" is a placeholder, skip
                        continue;
                    }
                    v
                }
                None => continue,
            };

            let bytes = if let Some(encoding) = signal.encoding {
                parse_value(value, encoding)
                    .map_err(|e| format!("Failed to parse secret signal '{}': {}", name, e))?
            } else {
                parse_value_auto(value)
                    .map_err(|e| format!("Failed to parse secret signal '{}': {}", name, e))?
            };

            let field_value = bytes_to_field(&bytes)?;
            signal_values.insert(name.clone(), field_value);
        }

        // Process public signals
        for (name, signal) in &program.public {
            // Skip output signals (value is None, empty string, or "?")
            let value = match &signal.value {
                Some(v) => {
                    if v.is_empty() || v == "?" {
                        // Empty string or "?" is treated as output signal
                        continue;
                    }
                    v
                }
                None => continue,  // Output signal, skip
            };

            let bytes = if let Some(encoding) = signal.encoding {
                parse_value(value, encoding)
                    .map_err(|e| format!("Failed to parse public signal '{}' (value={:?}, encoding={:?}): {}", name, signal.value, signal.encoding, e))?
            } else {
                parse_value_auto(value)
                    .map_err(|e| format!("Failed to parse public signal '{}' (value={:?}): {}", name, signal.value, e))?
            };

            let field_value = bytes_to_field(&bytes)?;
            signal_values.insert(name.clone(), field_value);
            public_signal_names.push(name.clone());
        }

        // Execute preprocessing operations (hashing, encoding, etc.)
        // Outputs become intermediate signals available in circuit
        if !program.preprocess.is_empty() {
            // Convert field elements back to bytes for preprocessing
            let mut signal_bytes: HashMap<String, Vec<u8>> = HashMap::new();

            for (name, field) in &signal_values {
                // Convert field element to bytes (little-endian, 32 bytes)
                let bytes = field.to_repr();
                signal_bytes.insert(name.clone(), bytes.as_ref().to_vec());
            }

            // Execute preprocessing operations
            // This may fail during verification when secret signals are not available
            // In that case, we skip preprocessing (the preprocessed values should already be in signal_values from verify context)
            if let Ok(preprocess_outputs) = crate::preprocess::execute_preprocess(
                &program.preprocess,
                &signal_bytes,
            ) {
                // Convert preprocessing outputs back to field elements
                for (name, output_bytes) in preprocess_outputs {
                    let field_value = bytes_to_field(&output_bytes)?;
                    signal_values.insert(name, field_value);
                }
            }
            // If preprocessing fails (e.g., during verification), we continue without it
            // The preprocessed signal values should be provided in the verify context
        }

        // Parse circuit statements
        let mut statements = Vec::new();
        for circuit_str in &program.circuit {
            // Check if this is an assignment (contains <==)
            if let Some(pos) = circuit_str.find("<==") {
                let name = circuit_str[..pos].trim().to_string();
                let expr_str = circuit_str[pos + 3..].trim();

                // Parse the expression
                let expression = parse_circuit(expr_str)
                    .map_err(|e| format!("Failed to parse assignment expression '{}': {}", expr_str, e))?;

                // Evaluate the expression to get the intermediate signal value
                // This may fail during verification when secret signals are not available
                // In that case, we skip storing the value but still add the statement
                if let Ok(value) = evaluate_expression(&expression, &signal_values) {
                    // Store the intermediate signal value for use in subsequent statements
                    signal_values.insert(name.clone(), value);
                }

                statements.push(Statement::Assignment {
                    name,
                    expression,
                });
            } else {
                // Regular expression
                let expression = parse_circuit(circuit_str)
                    .map_err(|e| format!("Failed to parse expression '{}': {}", circuit_str, e))?;

                statements.push(Statement::Expression(expression));
            }
        }

        // Evaluate circuit output from last statement
        let circuit_output = if let Some(last_stmt) = statements.last() {
            match last_stmt {
                Statement::Expression(expr) => evaluate_expression(expr, &signal_values).ok(),
                Statement::Assignment { expression, .. } => evaluate_expression(expression, &signal_values).ok(),
            }
        } else {
            None
        };

        let mut circuit = Self {
            expression: None,  // Use statements instead
            statements,
            signals: signal_values,
            public_signal_names,
            circuit_output,
            cached_max_bits: None,
            strategy: "auto".to_string(),
        };

        // Compute and cache max_bits from signal values
        circuit.cached_max_bits = circuit.compute_max_range_check_bits();

        Ok(circuit)
    }
}

// Wrapper types for different strategies
// Each type implements Circuit with its own configuration

/// Circuit with boolean operations support (AND, OR, NOT with comparison)
///
/// **Use for**: Circuits with boolean operations but no ordering comparisons
/// **Columns**: Fewer than full comparison config
/// **Proof size**: ~18-20 KB (between minimal and full)
///
/// # Example
///
/// ```ignore
/// // Circuit: (A == B) AND (C != 0) OR NOT D
/// // Or: (key1 == key2) AND (status != 0) OR NOT active
/// let circuit = Circuit::new(expr, signals, public);
/// let boolean = CircuitBoolean(circuit);
/// // Optimized for boolean operations and equality checks!
/// ```
#[derive(Clone)]
pub struct CircuitBoolean(pub Circuit);

// Implement Circuit for Boolean variant (boolean ops with minimal comparison)
impl PlonkCircuit<Fp> for CircuitBoolean {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        CircuitBoolean(self.0.without_witnesses())
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        CircuitConfig::configure_boolean(meta) // Use boolean config with minimal comparison
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        self.0.synthesize(config, layouter)
    }
}

/// Circuit with bit decomposition strategy (threshold=0, smallest proofs)
#[derive(Clone)]
pub struct CircuitBitD(pub Circuit);

/// Circuit with auto strategy (threshold=16, balanced)
#[derive(Clone)]
pub struct CircuitAuto(pub Circuit);

/// Circuit with lookup strategy (threshold=20, fastest proving)
#[derive(Clone)]
pub struct CircuitLookup(pub Circuit);

// Implement Circuit for BitD variant (threshold=0)
impl PlonkCircuit<Fp> for CircuitBitD {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        CircuitBitD(self.0.without_witnesses())
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        CircuitConfig::configure_with_strategy(meta, 0) // threshold=0: always bit decomposition
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        self.0.synthesize(config, layouter)
    }
}

// Implement Circuit for Auto variant (threshold=16)
impl PlonkCircuit<Fp> for CircuitAuto {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        CircuitAuto(self.0.without_witnesses())
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        CircuitConfig::configure_with_strategy(meta, 16) // threshold=16: balanced
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        self.0.synthesize(config, layouter)
    }
}

// Implement Circuit for Lookup variant (threshold=20)
impl PlonkCircuit<Fp> for CircuitLookup {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        CircuitLookup(self.0.without_witnesses())
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        CircuitConfig::configure_with_strategy(meta, 20) // threshold=20: prefer lookup
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        self.0.synthesize(config, layouter)
    }
}

impl PlonkCircuit<Fp> for Circuit {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            expression: self.expression.clone(),
            statements: self.statements.clone(),
            signals: HashMap::new(),
            public_signal_names: self.public_signal_names.clone(),
            circuit_output: None,  // Clear output (computed from witnesses)
            cached_max_bits: self.cached_max_bits,  // Preserve cached value!
            strategy: self.strategy.clone(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        CircuitConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // Conditionally load range check tables based on circuit operations
        // Only ordering comparisons (>, <, >=, <=) require range checks
        // Equality (==, !=) uses only is_zero gadget
        let max_bits = if self.uses_range_check_comparisons() {
            // Circuit uses ordering comparisons - load range check tables
            // Try to use cached value from original circuit, or default to 64 bits
            let bits = self.max_range_check_bits().unwrap_or(64);

            // CRITICAL: Always load tables if circuit uses ordering comparisons
            // This is required even in without_witnesses() for VK generation
            if let Some(comparison) = &config.comparison {
                comparison.range_check.load_up_to(&mut layouter, bits)?;
            } else {
                // Should never happen: circuit uses comparisons but config has no comparison support
                return Err(Error::Synthesis);
            }
            bits
        } else {
            // No ordering comparisons - skip range check table loading entirely
            // Use 64 as default (won't be used since no comparisons)
            64
        };

        // Create chip for circuit operations with the correct bit size
        let chip = CircuitChip::new(config.clone(), max_bits);

        // Synthesize main expression if present and capture result
        let circuit_result = if let Some(expr) = &self.expression {
            Some(chip.synthesize_expr(
                layouter.namespace(|| "circuit"),
                expr,
                &self.signals,
            )?)
        } else {
            None
        };

        // Synthesize statements if present and capture last result
        let mut last_stmt_result = None;
        for (idx, stmt) in self.statements.iter().enumerate() {
            match stmt {
                Statement::Assignment { name, expression } => {
                    last_stmt_result = Some(chip.synthesize_expr(
                        layouter.namespace(|| format!("assign_{}", name)),
                        expression,
                        &self.signals,
                    )?);
                }
                Statement::Expression(expression) => {
                    last_stmt_result = Some(chip.synthesize_expr(
                        layouter.namespace(|| format!("expr_{}", idx)),
                        expression,
                        &self.signals,
                    )?);
                }
            }
        }

        // Constrain public signals to instance column
        // Public signals are passed as instance inputs during proof creation/verification
        for (idx, signal_name) in self.public_signal_names.iter().enumerate() {
            // Get signal value if available (will be None for without_witnesses)
            let signal_value = self.signals.get(signal_name).copied();

            let cell = chip.assign_advice(
                layouter.namespace(|| format!("public_{}", signal_name)),
                config.advice[0],
                signal_value.map(Value::known).unwrap_or(Value::unknown()),
            )?;
            layouter.constrain_instance(cell.cell(), config.instance, idx)?;
        }

        // Constrain circuit output as additional public signal (last instance)
        // This ensures the proof commits to the actual circuit result
        let final_result = circuit_result.or(last_stmt_result);
        if let Some(result_cell) = final_result {
            let output_idx = self.public_signal_names.len();
            layouter.constrain_instance(result_cell.cell(), config.instance, output_idx)?;
        }

        Ok(())
    }
}

/// Chip for implementing circuit operations
struct CircuitChip {
    config: CircuitConfig,
    /// Maximum bit size for range checks (from circuit's cached_max_bits)
    max_bits: usize,
}

impl CircuitChip {
    fn new(config: CircuitConfig, max_bits: usize) -> Self {
        Self { config, max_bits }
    }

    /// Assign a value to an advice column
    fn assign_advice(
        &self,
        mut layouter: impl Layouter<Fp>,
        column: Column<Advice>,
        value: Value<Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "assign value",
            |mut region| {
                region.assign_advice(|| "value", column, 0, || value)
            },
        )
    }

    /// Add two values
    fn add(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "add",
            |mut region| {
                self.config.s_add.enable(&mut region, 0)?;

                let a_val = a.copy_advice(|| "lhs", &mut region, self.config.advice[0], 0)?;
                let b_val = b.copy_advice(|| "rhs", &mut region, self.config.advice[1], 0)?;

                let c_val = a_val.value().copied() + b_val.value().copied();
                region.assign_advice(|| "output", self.config.advice[2], 0, || c_val)
            },
        )
    }

    /// Multiply two values
    fn mul(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "mul",
            |mut region| {
                self.config.s_mul.enable(&mut region, 0)?;

                let a_val = a.copy_advice(|| "lhs", &mut region, self.config.advice[0], 0)?;
                let b_val = b.copy_advice(|| "rhs", &mut region, self.config.advice[1], 0)?;

                let c_val = a_val.value().copied() * b_val.value().copied();
                region.assign_advice(|| "output", self.config.advice[2], 0, || c_val)
            },
        )
    }

    /// Subtract b from a with proper constraint
    ///
    /// Implements: a - b = a + (-b)
    /// Uses negate() to compute -b with constraint, then add() with constraint
    fn sub(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // Step 1: Negate b with constraint (a * (-1) = -b)
        let neg_b = self.negate(layouter.namespace(|| "negate_b"), b)?;

        // Step 2: Add a + (-b) with constraint (a + (-b) = output)
        self.add(layouter.namespace(|| "add_a_neg_b"), a, &neg_b)
    }

    /// Divide a by b (multiply a by b^-1)
    ///
    /// Returns Error if b == 0 (division by zero)
    ///
    /// Uses mul gate to enforce: a * b^(-1) = output
    fn div(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "div",
            |mut region| {
                self.config.s_mul.enable(&mut region, 0)?;

                let a_val = a.copy_advice(|| "lhs", &mut region, self.config.advice[0], 0)?;
                let b_val = b.copy_advice(|| "rhs", &mut region, self.config.advice[1], 0)?;

                // Compute a * b^-1
                // Check for division by zero during witness computation
                let c_val = a_val.value().zip(b_val.value()).and_then(|(a, b)| {
                    // Check if b is zero
                    if *b == Fp::zero() {
                        // Division by zero - this will cause synthesis to fail
                        // because we return Value::unknown() which can't satisfy the constraint
                        return Value::unknown();
                    }

                    // Safe: b != 0, so invert() will succeed
                    let b_inv = b.invert().unwrap();
                    Value::known(*a * b_inv)
                });

                region.assign_advice(|| "result", self.config.advice[2], 0, || c_val)
            },
        )
    }

    /// Compare two values using range checks and is_zero gadget
    ///
    /// This uses the ComparisonChip which provides cryptographically sound comparisons:
    /// - Equality/Inequality: Uses is_zero gadget with full constraints
    /// - Greater/Less: Uses range checks + is_zero
    /// - GreaterEqual/LessEqual: Uses only range checks
    ///
    /// All comparisons return 1 (true) or 0 (false).
    fn compare(
        &self,
        mut layouter: impl Layouter<Fp>,
        op: &ComparisonOperator,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // Get comparison config (should always be Some if circuit uses comparisons)
        let comparison_config = self.config.comparison.as_ref()
            .ok_or(Error::Synthesis)?; // Error if minimal circuit tries to use comparisons

        // Create comparison chip
        let chip = ComparisonChip::new(comparison_config.clone());

        // Use the bit size that was determined during circuit construction
        // This ensures we use the correct lookup table (8, 16, 32, or 64 bits)
        let bits = self.max_bits;

        match op {
            ComparisonOperator::Equal => {
                chip.is_equal(layouter.namespace(|| "is_equal"), a, b)
            }
            ComparisonOperator::NotEqual => {
                chip.is_not_equal(layouter.namespace(|| "is_not_equal"), a, b)
            }
            ComparisonOperator::Greater => {
                chip.is_greater(layouter.namespace(|| "is_greater"), a, b, bits)
            }
            ComparisonOperator::Less => {
                chip.is_less(layouter.namespace(|| "is_less"), a, b, bits)
            }
            ComparisonOperator::GreaterEqual => {
                chip.is_greater_or_equal(layouter.namespace(|| "is_greater_or_equal"), a, b, bits)
            }
            ComparisonOperator::LessEqual => {
                chip.is_less_or_equal(layouter.namespace(|| "is_less_or_equal"), a, b, bits)
            }
        }
    }

    /// Boolean AND: both values non-zero -> 1, else 0
    ///
    /// Uses is_zero gadget to convert to bool, then multiplies with constraint
    fn boolean_and(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // Get comparison config (should always be Some if circuit uses boolean ops)
        let comparison_config = self.config.comparison.as_ref()
            .ok_or(Error::Synthesis)?; // Error if minimal circuit tries to use boolean ops

        let chip = ComparisonChip::new(comparison_config.clone());

        // Convert a to boolean: is_not_zero(a) = NOT(is_zero(a))
        let a_is_zero = chip.is_zero(layouter.namespace(|| "a_is_zero"), a)?;
        let a_bool = chip.is_zero(layouter.namespace(|| "a_to_bool"), &a_is_zero)?;

        // Convert b to boolean: is_not_zero(b) = NOT(is_zero(b))
        let b_is_zero = chip.is_zero(layouter.namespace(|| "b_is_zero"), b)?;
        let b_bool = chip.is_zero(layouter.namespace(|| "b_to_bool"), &b_is_zero)?;

        // Multiply bool values: bool_a * bool_b = output
        // Uses mul gate with constraint
        self.mul(layouter.namespace(|| "and_mul"), &a_bool, &b_bool)
    }

    /// Boolean OR: any value non-zero -> 1, else 0
    ///
    /// Uses De Morgan's law: OR(a, b) = NOT(NOT(a) AND NOT(b))
    /// All operations use proper constraints
    fn boolean_or(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
        b: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // Step 1: NOT a (using is_zero)
        let not_a = self.boolean_not(layouter.namespace(|| "not_a"), a)?;

        // Step 2: NOT b (using is_zero)
        let not_b = self.boolean_not(layouter.namespace(|| "not_b"), b)?;

        // Step 3: AND(NOT a, NOT b) (using boolean_and with constraints)
        let both_false = self.boolean_and(layouter.namespace(|| "and_not"), &not_a, &not_b)?;

        // Step 4: NOT(both_false) = OR(a, b)
        self.boolean_not(layouter.namespace(|| "not_both_false"), &both_false)
    }

    /// Boolean NOT: 0 -> 1, non-zero -> 0
    ///
    /// Uses is_zero gadget with proper constraints
    fn boolean_not(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        // Get comparison config (should always be Some if circuit uses boolean ops)
        let comparison_config = self.config.comparison.as_ref()
            .ok_or(Error::Synthesis)?; // Error if minimal circuit tries to use boolean ops

        // NOT is exactly is_zero!
        // is_zero(a) returns 1 if a == 0, else 0
        let chip = ComparisonChip::new(comparison_config.clone());
        chip.is_zero(layouter.namespace(|| "boolean_not"), a)
    }

    /// Negate a value with proper constraint
    ///
    /// Uses mul gate to enforce: a * (-1) = output
    fn negate(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "negate",
            |mut region| {
                // Enable mul gate to enforce constraint: a * (-1) = output
                self.config.s_mul.enable(&mut region, 0)?;

                // Copy a to advice[0]
                let a_val = a.copy_advice(|| "operand", &mut region, self.config.advice[0], 0)?;

                // Assign -1 to advice[1]
                region.assign_advice(
                    || "minus_one",
                    self.config.advice[1],
                    0,
                    || Value::known(-Fp::one()),
                )?;

                // Compute output = -a
                let output_val = a_val.value().map(|a| -a);

                // Assign output to advice[2]
                // The mul gate will enforce: a * (-1) = output
                region.assign_advice(|| "neg_output", self.config.advice[2], 0, || output_val)
            },
        )
    }

    /// Recursively synthesize an expression
    fn synthesize_expr(
        &self,
        mut layouter: impl Layouter<Fp>,
        expr: &Expression,
        signals: &HashMap<String, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        match expr {
            Expression::Variable(name) => {
                // Get value if available (will be None for without_witnesses)
                let value = signals.get(name).copied()
                    .map(Value::known)
                    .unwrap_or(Value::unknown());
                self.assign_advice(
                    layouter.namespace(|| format!("var_{}", name)),
                    self.config.advice[0],
                    value,
                )
            }

            Expression::Constant(s) => {
                // Parse constant with arbitrary precision support
                let field_value = parse_constant_to_field(s)
                    .map_err(|_| Error::Synthesis)?;
                self.assign_advice(
                    layouter.namespace(|| format!("const_{}", s)),
                    self.config.advice[0],
                    Value::known(field_value),
                )
            }

            Expression::Boolean(b) => {
                let value = if *b { Fp::one() } else { Fp::zero() };
                self.assign_advice(
                    layouter.namespace(|| format!("bool_{}", b)),
                    self.config.advice[0],
                    Value::known(value),
                )
            }

            Expression::BinaryOp { op, left, right } => {
                let l = self.synthesize_expr(layouter.namespace(|| "left"), left, signals)?;
                let r = self.synthesize_expr(layouter.namespace(|| "right"), right, signals)?;

                match op {
                    BinaryOperator::Add => self.add(layouter.namespace(|| "add"), &l, &r),
                    BinaryOperator::Sub => self.sub(layouter.namespace(|| "sub"), &l, &r),
                    BinaryOperator::Mul => self.mul(layouter.namespace(|| "mul"), &l, &r),
                    BinaryOperator::Div => self.div(layouter.namespace(|| "div"), &l, &r),
                }
            }

            Expression::UnaryOp { op, operand } => {
                let val = self.synthesize_expr(layouter.namespace(|| "operand"), operand, signals)?;

                match op {
                    UnaryOperator::Neg => self.negate(layouter.namespace(|| "neg"), &val),
                    UnaryOperator::Not => self.boolean_not(layouter.namespace(|| "not"), &val),
                }
            }

            Expression::Comparison { op, left, right } => {
                let l = self.synthesize_expr(layouter.namespace(|| "left"), left, signals)?;
                let r = self.synthesize_expr(layouter.namespace(|| "right"), right, signals)?;

                self.compare(layouter.namespace(|| "compare"), op, &l, &r)
            }

            Expression::BooleanOp { op, left, right } => {
                let l = self.synthesize_expr(layouter.namespace(|| "left"), left, signals)?;
                let r = self.synthesize_expr(layouter.namespace(|| "right"), right, signals)?;

                match op {
                    BooleanOperator::And => self.boolean_and(layouter.namespace(|| "and"), &l, &r),
                    BooleanOperator::Or => self.boolean_or(layouter.namespace(|| "or"), &l, &r),
                }
            }
        }
    }
}

/// Parse constant (decimal string) to field element with arbitrary precision
///
/// Supports constants of any size by reducing modulo the Pallas field modulus.
///
/// # Arguments
///
/// * `value` - Decimal string representation (e.g., "123", "999999999999999999...")
///
/// # Returns
///
/// Field element reduced modulo Pallas field
///
/// # Example
///
/// ```ignore
/// let field = parse_constant_to_field("12345")?;
/// let large = parse_constant_to_field("999999999999999999999999")?;
/// ```
fn parse_constant_to_field(value: &str) -> Result<Fp, String> {
    // Parse decimal string as BigUint
    let num = BigUint::from_str_radix(value, 10)
        .map_err(|_| format!("Invalid decimal constant: {}", value))?;

    // Convert to big-endian bytes
    let bytes = num.to_bytes_be();

    // Use bytes_to_field() for conversion
    bytes_to_field(&bytes)
}

/// Convert bytes to field element with arbitrary precision
///
/// Supports values of any size by reducing modulo the Pallas field modulus.
/// This allows working with large values like Solana addresses (32 bytes).
///
/// # Arguments
///
/// * `bytes` - Big-endian byte representation of the value
///
/// # Returns
///
/// Field element reduced modulo Pallas field
///
/// # Example
///
/// ```ignore
/// // Small value
/// let bytes = vec![0x01, 0x23];  // 291 in big-endian
/// let field = bytes_to_field(&bytes)?;
///
/// // Large value (Solana address - 32 bytes)
/// let bytes = vec![0x12; 32];
/// let field = bytes_to_field(&bytes)?;  // Automatically reduced modulo field
/// ```
fn bytes_to_field(bytes: &[u8]) -> Result<Fp, String> {
    // Handle empty bytes
    if bytes.is_empty() {
        return Ok(Fp::zero());
    }

    // Convert bytes to BigUint (big-endian input)
    let num = BigUint::from_bytes_be(bytes);

    // Pallas field modulus: p = 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001
    let modulus = BigUint::parse_bytes(
        b"40000000000000000000000000000000224698fc094cf91b992d30ed00000001",
        16
    ).expect("Valid Pallas modulus");

    // Reduce modulo p (automatically handles values larger than field)
    let reduced = num % modulus;

    // Convert to little-endian bytes (Fp internal representation is little-endian)
    let mut le_bytes = reduced.to_bytes_le();

    // Pad to 32 bytes if needed (Pallas field elements are 32 bytes)
    le_bytes.resize(32, 0);

    // Create Fp byte representation
    let mut repr = [0u8; 32];
    repr.copy_from_slice(&le_bytes[..32]);

    // Convert to Fp using from_repr
    // This should always succeed since we reduced modulo field
    Fp::from_repr(repr)
        .into_option()
        .ok_or_else(|| "Failed to convert to field element (should never happen)".to_string())
}

/// Helper to evaluate expressions (for witness generation)
pub fn evaluate_expression(
    expr: &Expression,
    signals: &HashMap<String, Fp>,
) -> Result<Fp, String> {
    match expr {
        Expression::Variable(name) => {
            signals.get(name)
                .cloned()
                .ok_or_else(|| format!("Variable '{}' not found", name))
        }

        Expression::Constant(value) => {
            // Parse constant with arbitrary precision support
            parse_constant_to_field(value)
        }

        Expression::Boolean(b) => {
            Ok(if *b { Fp::one() } else { Fp::zero() })
        }

        Expression::BinaryOp { op, left, right } => {
            let l = evaluate_expression(left, signals)?;
            let r = evaluate_expression(right, signals)?;

            match op {
                BinaryOperator::Add => Ok(l + r),
                BinaryOperator::Sub => Ok(l - r),
                BinaryOperator::Mul => Ok(l * r),
                BinaryOperator::Div => {
                    let r_inv = r.invert().unwrap_or(Fp::zero());
                    Ok(l * r_inv)
                }
            }
        }

        Expression::UnaryOp { op, operand } => {
            let val = evaluate_expression(operand, signals)?;

            match op {
                UnaryOperator::Neg => Ok(-val),
                UnaryOperator::Not => {
                    // NOT: 0 -> 1, any non-zero -> 0
                    Ok(if val == Fp::zero() { Fp::one() } else { Fp::zero() })
                }
            }
        }

        Expression::Comparison { op, left, right } => {
            let l = evaluate_expression(left, signals)?;
            let r = evaluate_expression(right, signals)?;

            // Convert to u64 for comparison
            let l_val = field_to_u64(&l);
            let r_val = field_to_u64(&r);

            let result = match op {
                ComparisonOperator::Greater => l_val > r_val,
                ComparisonOperator::Less => l_val < r_val,
                ComparisonOperator::Equal => l_val == r_val,
                ComparisonOperator::GreaterEqual => l_val >= r_val,
                ComparisonOperator::LessEqual => l_val <= r_val,
                ComparisonOperator::NotEqual => l_val != r_val,
            };

            Ok(if result { Fp::one() } else { Fp::zero() })
        }

        Expression::BooleanOp { op, left, right } => {
            let l = evaluate_expression(left, signals)?;
            let r = evaluate_expression(right, signals)?;

            // Treat any non-zero as true
            let l_bool = l != Fp::zero();
            let r_bool = r != Fp::zero();

            let result = match op {
                BooleanOperator::And => l_bool && r_bool,
                BooleanOperator::Or => l_bool || r_bool,
            };

            Ok(if result { Fp::one() } else { Fp::zero() })
        }
    }
}

/// Helper to convert field element to u64 (for comparisons)
fn field_to_u64(f: &Fp) -> u64 {
    let bytes = f.to_repr();
    let mut value = 0u64;
    for i in 0..8.min(bytes.as_ref().len()) {
        value |= (bytes.as_ref()[i] as u64) << (i * 8);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_arithmetic() {
        let mut signals = HashMap::new();
        signals.insert("A".to_string(), Fp::from(10));
        signals.insert("B".to_string(), Fp::from(20));

        let expr = Expression::add(
            Expression::var("A"),
            Expression::var("B"),
        );

        let result = evaluate_expression(&expr, &signals).unwrap();
        assert_eq!(result, Fp::from(30));
    }

    #[test]
    fn test_evaluate_comparison() {
        let mut signals = HashMap::new();
        signals.insert("A".to_string(), Fp::from(10));
        signals.insert("B".to_string(), Fp::from(20));

        // A < B = 1 (true)
        let expr = Expression::compare(
            ComparisonOperator::Less,
            Expression::var("A"),
            Expression::var("B"),
        );

        assert_eq!(evaluate_expression(&expr, &signals).unwrap(), Fp::one());
    }

    #[test]
    fn test_evaluate_boolean() {
        let mut signals = HashMap::new();
        signals.insert("A".to_string(), Fp::from(1));
        signals.insert("B".to_string(), Fp::from(4));

        // 1 AND 4 = 1 (both non-zero)
        let expr = Expression::and(
            Expression::var("A"),
            Expression::var("B"),
        );

        assert_eq!(evaluate_expression(&expr, &signals).unwrap(), Fp::one());
    }

    #[test]
    fn test_bytes_to_field_small_value() {
        // Test small value (< 8 bytes)
        let bytes = vec![0x01, 0x23]; // 291 in big-endian
        let field = bytes_to_field(&bytes).unwrap();

        // Expected: 0x0123 = 291
        assert_eq!(field, Fp::from(291));

        // Test zero
        let bytes = vec![0x00];
        let field = bytes_to_field(&bytes).unwrap();
        assert_eq!(field, Fp::zero());

        // Test empty
        let bytes = vec![];
        let field = bytes_to_field(&bytes).unwrap();
        assert_eq!(field, Fp::zero());
    }

    #[test]
    fn test_bytes_to_field_large_value() {
        // Test large value (32 bytes - Solana address size)
        let bytes = vec![0x12; 32]; // 32 bytes of 0x12

        // Should not fail - automatically reduced modulo field
        let field = bytes_to_field(&bytes).unwrap();

        // Verify it's not zero (reduction should give non-zero value)
        assert_ne!(field, Fp::zero());

        // Test maximum field value (just under modulus)
        // Pallas modulus: 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001
        let mut max_bytes = vec![0xFF; 32];
        let field = bytes_to_field(&max_bytes).unwrap();
        // Should succeed with reduced value
        assert_ne!(field, Fp::zero());

        // Test that two different large values produce different field elements
        max_bytes[0] = 0xFE;
        let field2 = bytes_to_field(&max_bytes).unwrap();
        assert_ne!(field, field2);
    }

    #[test]
    fn test_bytes_to_field_solana_address_equality() {
        use crate::encoding::{parse_value, ValueEncoding};

        // Real Solana address (Base58)
        let address1 = "9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM";
        let bytes1 = parse_value(address1, ValueEncoding::Base58).unwrap();
        let field1 = bytes_to_field(&bytes1).unwrap();

        // Same address should produce same field element
        let bytes2 = parse_value(address1, ValueEncoding::Base58).unwrap();
        let field2 = bytes_to_field(&bytes2).unwrap();
        assert_eq!(field1, field2);

        // Different address should produce different field element
        let address3 = "So11111111111111111111111111111111111111112";
        let bytes3 = parse_value(address3, ValueEncoding::Base58).unwrap();
        let field3 = bytes_to_field(&bytes3).unwrap();
        assert_ne!(field1, field3);
    }

    #[test]
    fn test_parse_constant_small_value() {
        // Test small decimal constant
        let field = parse_constant_to_field("12345").unwrap();
        assert_eq!(field, Fp::from(12345));

        // Test zero
        let field = parse_constant_to_field("0").unwrap();
        assert_eq!(field, Fp::zero());

        // Test one
        let field = parse_constant_to_field("1").unwrap();
        assert_eq!(field, Fp::one());
    }

    #[test]
    fn test_parse_constant_large_value() {
        // Test value larger than u64::MAX
        let large = "999999999999999999999999999999";  // Much larger than u64::MAX
        let field = parse_constant_to_field(large).unwrap();

        // Should succeed (reduced modulo field)
        assert_ne!(field, Fp::zero());

        // Test that different large values produce different results
        let large2 = "999999999999999999999999999998";
        let field2 = parse_constant_to_field(large2).unwrap();
        assert_ne!(field, field2);
    }

    #[test]
    fn test_parse_constant_error() {
        // Test invalid constant (not a number)
        assert!(parse_constant_to_field("not_a_number").is_err());
        assert!(parse_constant_to_field("12.34").is_err());  // No decimals
        assert!(parse_constant_to_field("0x123").is_err());  // No hex prefix
    }

    #[test]
    fn test_from_program_with_preprocess() {
        use crate::api::Program;

        // Create program with preprocessing
        // Program: hash<==sha256(A{%x}|B{%x}); hash == threshold
        let program_str = "1/A:255,B:16/-/hash<==sha256(A{%x}|B{%x})/hash==threshold";
        let program = Program::from_zircon(program_str).unwrap();

        // Build circuit from program
        let circuit = Circuit::from_program(&program).unwrap();

        // Verify that preprocessing result (hash) is in signals
        assert!(circuit.signals.contains_key("hash"));

        // Verify hash value is not zero (SHA-256 of "ff10")
        let hash_value = circuit.signals.get("hash").unwrap();
        assert_ne!(*hash_value, Fp::zero());

        // Verify that circuit statements were parsed correctly
        assert_eq!(circuit.statements.len(), 1);

        // Verify no expression (using statements instead)
        assert!(circuit.expression.is_none());
    }

    #[test]
    fn test_from_program_with_multiple_preprocess() {
        use crate::api::Program;

        // Program with multiple preprocessing steps
        // First: encoded <== hex_encode(A)
        // Second: hash <== sha256(encoded)
        // Circuit: A > 100
        let zircon = "1/A:255/-/encoded<==hex_encode(A);hash<==sha256(encoded)/A>100";
        let program = Program::from_zircon(zircon).unwrap();

        // Build circuit
        let circuit = Circuit::from_program(&program).unwrap();

        // Verify both intermediate signals exist
        assert!(circuit.signals.contains_key("encoded"));
        assert!(circuit.signals.contains_key("hash"));
        assert!(circuit.signals.contains_key("A"));

        // Verify values are not zero
        assert_ne!(*circuit.signals.get("encoded").unwrap(), Fp::zero());
        assert_ne!(*circuit.signals.get("hash").unwrap(), Fp::zero());

        // Verify A has correct value
        assert_eq!(*circuit.signals.get("A").unwrap(), Fp::from(255));
    }

    #[test]
    fn test_full_integration_pipe_and_or() {
        use crate::api::Program;

        // Test complete workflow: parsing → preprocessing with | → circuit with ||
        // Program: hash<==sha256(A{%x}|B{%d}); (hash>100)||(A<10)
        let program_str = "1/A:255,B:1000/-/hash<==sha256(A{%x}|B{%d})/(hash>100)||(A<10)";
        let program = Program::from_zircon(program_str).unwrap();

        // Build circuit from program
        let circuit = Circuit::from_program(&program).unwrap();

        // Verify preprocessing worked - hash signal exists
        assert!(circuit.signals.contains_key("hash"));
        let hash_value = circuit.signals.get("hash").unwrap();
        assert_ne!(*hash_value, Fp::zero());

        // Verify circuit statement was parsed (contains ||)
        assert_eq!(circuit.statements.len(), 1);

        // Verify all input signals are present
        assert!(circuit.signals.contains_key("A"));
        assert!(circuit.signals.contains_key("B"));
        assert_eq!(*circuit.signals.get("A").unwrap(), Fp::from(255));
        assert_eq!(*circuit.signals.get("B").unwrap(), Fp::from(1000));
    }

    #[test]
    fn test_complex_preprocessing_with_concatenation() {
        use crate::api::Program;

        // Test multiple values concatenated with |
        let program_str = "1/A:10,B:20,C:30/-/hash<==sha256(A{%d}|B{%d}|C{%d})/hash>0";
        let program = Program::from_zircon(program_str).unwrap();

        let circuit = Circuit::from_program(&program).unwrap();

        // Verify hash exists and is non-zero
        assert!(circuit.signals.contains_key("hash"));
        assert_ne!(*circuit.signals.get("hash").unwrap(), Fp::zero());

        // Verify input values
        assert_eq!(*circuit.signals.get("A").unwrap(), Fp::from(10));
        assert_eq!(*circuit.signals.get("B").unwrap(), Fp::from(20));
        assert_eq!(*circuit.signals.get("C").unwrap(), Fp::from(30));
    }
}