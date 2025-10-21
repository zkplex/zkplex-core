//! API module
//!
//! This module contains the JSON API types and structures.

mod types;
pub mod program;
pub mod core;
pub mod prove_helpers;
pub mod layout;

// Re-export types from types module (for JSON API)
pub use types::{
    Signal, ProveRequest, ProveResponse,
    VerifyRequest, VerifyResponse, ErrorResponse,
    EstimateResponse, DebugInfo, PublicSignal, VerifyContext,
    PROOF_VERSION, // Re-export proof version constant
};

// Re-export Program type (Signal within program is kept internal)
pub use program::Program;

// Re-export prove helpers
pub use prove_helpers::{apply_signal_overrides, program_to_prove_request};

// Re-export layout types
pub use layout::{
    CircuitLayout, CircuitParameters, RowLayout, ResourceRequirements,
    SignalInformation, OperationBreakdown, ColumnConfiguration,
    GateBreakdown, ArithmeticGates, ComparisonGates, PreprocessingGates,
    LookupTableInfo, MemoryUsage, ProverMemory, VerifierMemory,
    ComplexityAnalysis,
};