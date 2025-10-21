//! Circuit layout types for API responses
//!
//! This module defines the data structures for circuit layout information
//! that can be serialized to JSON and returned via WASM API.

use serde::{Deserialize, Serialize};

/// Complete circuit layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitLayout {
    /// Circuit expression
    pub circuit: String,

    /// Preprocessing expressions (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preprocess: Option<String>,

    /// Strategy used
    pub strategy: String,

    /// Strategy description
    pub strategy_description: String,

    /// Circuit parameters
    pub parameters: CircuitParameters,

    /// Row layout breakdown
    pub row_layout: RowLayout,

    /// Resource requirements
    pub resources: ResourceRequirements,

    /// Signal information
    pub signals: SignalInformation,

    /// Operation breakdown
    pub operations: OperationBreakdown,

    /// Column configuration
    pub columns: ColumnConfiguration,

    /// Gate type breakdown
    pub gates: GateBreakdown,

    /// Lookup table information (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup_tables: Option<LookupTableInfo>,

    /// Memory usage estimates
    pub memory: MemoryUsage,

    /// Complexity analysis
    pub complexity: ComplexityAnalysis,
}

/// Circuit parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitParameters {
    /// k parameter (circuit size = 2^k)
    pub k: u32,

    /// Total rows (2^k)
    pub total_rows: u64,

    /// Range check bits
    pub max_bits: usize,
}

/// Row layout breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowLayout {
    /// Range check table rows (if any)
    pub range_table_rows: u64,

    /// Range check table percentage
    pub range_table_percent: f64,

    /// Circuit gate rows
    pub circuit_rows: u64,

    /// Circuit rows percentage
    pub circuit_percent: f64,

    /// Unused rows
    pub unused_rows: u64,

    /// Unused percentage
    pub unused_percent: f64,

    /// Total used rows
    pub used_rows: u64,

    /// Total utilization percentage
    pub utilization_percent: f64,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Proving key size in bytes
    pub params_size_bytes: usize,

    /// Proving key size in KB
    pub params_size_kb: usize,

    /// Proof size in bytes
    pub proof_size_bytes: usize,

    /// Proof size in KB
    pub proof_size_kb: f64,

    /// Verification key size in bytes
    pub vk_size_bytes: usize,

    /// Verification key size in KB
    pub vk_size_kb: f64,
}

/// Signal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInformation {
    /// Total number of signals
    pub total: usize,

    /// Number of secret signals
    pub secret_count: usize,

    /// Secret signal names (up to 5)
    pub secret_names: Vec<String>,

    /// Number of secret signals not shown
    pub secret_more: usize,

    /// Number of public signals
    pub public_count: usize,

    /// Public signal names (up to 5)
    pub public_names: Vec<String>,

    /// Number of public signals not shown
    pub public_more: usize,
}

/// Operation breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationBreakdown {
    /// Total operations
    pub total: usize,

    /// Arithmetic operations count
    pub arithmetic_count: usize,

    /// Arithmetic operations percentage
    pub arithmetic_percent: f64,

    /// Comparison operations count
    pub comparison_count: usize,

    /// Comparison operations percentage
    pub comparison_percent: f64,

    /// Preprocessing operations count
    pub preprocess_count: usize,

    /// Preprocessing operations percentage
    pub preprocess_percent: f64,
}

/// Column configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfiguration {
    /// Total columns
    pub total: usize,

    /// Advice columns
    pub advice: usize,

    /// Instance columns
    pub instance: usize,

    /// Selector columns
    pub selector: usize,

    /// Fixed columns
    pub fixed: usize,
}

/// Gate type breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateBreakdown {
    /// Arithmetic gates
    pub arithmetic: ArithmeticGates,

    /// Comparison gates (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison: Option<ComparisonGates>,

    /// Preprocessing gates (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preprocessing: Option<PreprocessingGates>,
}

/// Arithmetic gate details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticGates {
    /// Addition/Subtraction count (estimate)
    pub addition_subtraction: usize,

    /// Multiplication/Division count (estimate)
    pub multiplication_division: usize,
}

/// Comparison gate details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonGates {
    /// Ordering comparison count
    pub ordering_count: usize,

    /// Uses range checks
    pub uses_range_checks: bool,
}

/// Preprocessing gate details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingGates {
    /// Hash operation count
    pub hash_operations: usize,
}

/// Lookup table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTableInfo {
    /// Bit size of table (8, 16, or mixed)
    pub bit_size: String,

    /// Number of rows in table
    pub table_rows: u64,

    /// Table overhead as percentage of circuit
    pub overhead_percent: f64,

    /// Total number of lookups
    pub total_lookups: usize,
}

/// Memory usage estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Prover memory requirements
    pub prover: ProverMemory,

    /// Verifier memory requirements
    pub verifier: VerifierMemory,
}

/// Prover memory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProverMemory {
    /// Proving params in MB
    pub params_mb: f64,

    /// Witness data in MB
    pub witness_mb: f64,

    /// Number of signals
    pub signal_count: usize,

    /// Working memory in MB
    pub working_mb: f64,

    /// Total peak memory in MB
    pub total_mb: f64,
}

/// Verifier memory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierMemory {
    /// Verification key in KB
    pub vk_kb: f64,

    /// Proof data in KB
    pub proof_kb: f64,

    /// Working memory in KB
    pub working_kb: f64,

    /// Total peak memory in KB
    pub total_kb: f64,
}

/// Complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    /// Overall complexity level
    pub overall: String,

    /// Prover time estimate
    pub prover_time: String,

    /// Verifier time estimate (always fast)
    pub verifier_time: String,

    /// Optimization suggestions
    pub optimization_suggestions: Vec<String>,
}