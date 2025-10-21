//! JSON API structures for prove/verify operations
//!
//! This module defines the JSON request/response formats for the WASM API.

use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use crate::encoding::{ValueEncoding, parse_value, parse_value_auto};
use crate::circuit::Strategy;

/// Current API version for proof format
pub const PROOF_VERSION: u32 = 1;

/// Signal definition with value and visibility
///
/// # Value Formats
///
/// The `value` field can be provided in multiple formats:
///
/// ## Decimal (default)
/// ```json
/// { "value": "12345", "public": false }
/// ```
///
/// ## Hexadecimal (Ethereum addresses, hashes)
/// ```json
/// { "value": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb", "encoding": "hex" }
/// ```
///
/// ## Base58 (Solana/Bitcoin keys)
/// ```json
/// { "value": "9aE476sH92Vc7DMCzKNgWUiQ6UdC2DXf9v", "encoding": "base58" }
/// ```
///
/// ## Base64 (Universal encoding)
/// ```json
/// { "value": "SGVsbG8gV29ybGQ=", "encoding": "base64" }
/// ```
///
/// ## Output Signal (computed value)
/// For output signals, omit the `value` field:
/// ```json
/// { "public": true }
/// ```
/// The value will be computed during proof generation and included in the response.
///
/// ## Auto-detection
/// If `encoding` is omitted, the format is auto-detected:
/// - Starts with "0x" → hex
/// - All digits → decimal
/// - Contains base64 chars (+/=) → base64
/// - Otherwise → base58 or decimal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Value in the specified encoding format
    /// Optional for output signals (will be computed during proof generation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Encoding format (default: auto-detect or decimal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<ValueEncoding>,

    /// Whether this signal is public (default: false = secret/witness)
    #[serde(default)]
    pub public: bool,
}

/// Request to create a ZKP proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveRequest {
    /// Preprocessing statements (executed before circuit)
    /// Format: `name <== operation(args)`
    /// Examples: `hash <== sha256(A)`, `encoded <== base64(data)`
    /// Optional - for advanced cryptographic operations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocess: Vec<String>,

    /// Circuit statements (last one is output)
    /// Can be a single statement or multiple statements
    /// Examples: `"(A + B) > C"` or `["A > B", "B < 1000", "A + B == C"]`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub circuit: Vec<String>,

    /// Signal assignments (variable name -> signal)
    pub signals: IndexMap<String, Signal>,

    /// Range check strategy (optional, default: "auto")
    /// - "auto": Balanced (chooses based on range)
    /// - "lookup": Prefer lookup tables (faster proving)
    /// - "bitd": Prefer bit decomposition (smaller proofs)
    /// - "boolean": Base strategy (no range comparisons)
    #[serde(default)]
    pub strategy: Strategy,
}

/// Public signal value with optional encoding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicSignal {
    /// Signal value as string
    pub value: String,

    /// Original encoding format (if specified during proof generation)
    /// If None, the value format should be auto-detected during verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<ValueEncoding>,
}

/// Debug information for proof generation
/// Contains human-readable version of verification context plus warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Preprocessing statements (if any)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocess: Vec<String>,

    /// Circuit statements
    pub circuit: Vec<String>,

    /// Circuit parameter k (determines circuit size as 2^k rows)
    pub k: u32,

    /// Range check strategy used for proof generation
    pub strategy: Strategy,

    /// Maximum bits for range check table (if circuit uses range checks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bits: Option<usize>,

    /// Names of secret signals used in the circuit
    pub secret_signals: Vec<String>,

    /// Name of the output signal (public signal that receives the computed result)
    pub output_signal: String,

    /// Optional warnings about privacy or security
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// Response from proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveResponse {
    /// Proof format version (current: 1)
    #[serde(default = "default_version")]
    pub version: u32,

    /// Proof data (base85-encoded)
    pub proof: String,

    /// Verification context (base85-encoded JSON)
    /// Contains circuit, strategy, k, and secret signal names needed to regenerate VK
    pub verify_context: String,

    /// Public signal values with encoding information
    pub public_signals: IndexMap<String, PublicSignal>,

    /// Debug information (human-readable verification context + warnings)
    /// Optional - only included for debugging/logging purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugInfo>,
}

fn default_version() -> u32 {
    PROOF_VERSION
}

/// Verification context that contains all parameters needed for proof verification
///
/// This structure is serialized to JSON and then base85-encoded in the proof file.
/// It bundles all information needed to reconstruct the circuit during verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyContext {
    /// Circuit parameter k (determines circuit size as 2^k rows)
    pub k: u32,

    /// Preprocessing statements (can be empty)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocess: Vec<String>,

    /// Circuit statements
    pub circuit: Vec<String>,

    /// Range check strategy used (auto, boolean, lookup, or bitd)
    pub strategy: Strategy,

    /// Names of all secret (private) signals used during proof generation
    /// These are needed to reconstruct the circuit with the same structure
    pub secret_signals: Vec<String>,

    /// Name of the output signal (the public signal whose value was computed during proof generation)
    pub output_signal: String,

    /// Cached maximum bits for range check table (if circuit uses range checks)
    /// This is needed to reconstruct the same circuit constraints during verification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_max_bits: Option<usize>,
}

/// Request to verify a ZKP proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    /// Proof format version (current: 1)
    #[serde(default = "default_version")]
    pub version: u32,

    /// Proof data (base85-encoded)
    pub proof: String,

    /// Verification context (base85-encoded JSON)
    /// Contains circuit, strategy, k, and secret signal names needed to regenerate VK
    pub verify_context: String,

    /// Public signal values with optional encoding information
    /// Can be simple strings (for backward compatibility) or PublicSignal objects
    pub public_signals: IndexMap<String, PublicSignal>,
}

/// Response from proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Whether the proof is valid
    pub valid: bool,

    /// Optional error message if verification failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,

    /// Optional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Response from circuit estimation
///
/// Provides hardware-independent metrics about circuit requirements.
/// These metrics do not depend on the specific machine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateResponse {
    /// Required k parameter for Halo2 (circuit fits in 2^k rows)
    pub k: u32,

    /// Total number of rows available (2^k)
    pub total_rows: u64,

    /// Estimated number of rows used by the circuit
    pub estimated_rows: u32,

    /// Number of arithmetic operations in the circuit
    pub operation_count: u32,

    /// Number of comparison operations (these are expensive)
    pub comparison_count: u32,

    /// Number of preprocessing statements (hash operations, etc.)
    pub preprocess_count: u32,

    /// Estimated Params size in bytes (contains 2^k curve points)
    /// This is approximately: 2^k * 32 bytes per point
    pub params_size_bytes: u64,

    /// Estimated proof size in bytes (grows logarithmically with k)
    /// Approximately: 2KB + (k * 100 bytes)
    pub proof_size_bytes: u64,

    /// Estimated verification key size in bytes
    /// Depends on circuit complexity, roughly: 1KB + (columns * 32 bytes)
    pub vk_size_bytes: u64,

    /// Circuit complexity description
    pub complexity: String,
}

impl ProveRequest {
    /// Validate the prove request
    ///
    /// Checks that:
    /// - Circuit is not empty
    /// - At least one signal is provided
    /// - All signal values can be parsed according to their encoding
    pub fn validate(&self) -> Result<(), String> {
        if self.circuit.is_empty() {
            return Err("Circuit cannot be empty".to_string());
        }

        if self.signals.is_empty() {
            return Err("At least one signal (output) must be provided".to_string());
        }

        // Validate that all signal values can be parsed according to their encoding
        // Skip output signals (no value)
        for (name, signal) in &self.signals {
            // Skip validation for output signals (value is None)
            let value = match &signal.value {
                Some(v) => v,
                None => {
                    // Output signal - must be public
                    if !signal.public {
                        return Err(format!(
                            "Output signal '{}' must be public",
                            name
                        ));
                    }
                    continue;
                }
            };

            let output = if let Some(encoding) = signal.encoding {
                // Use explicit encoding
                parse_value(value, encoding)
            } else {
                // Auto-detect encoding
                parse_value_auto(value)
            };

            if let Err(e) = output {
                return Err(format!(
                    "Signal '{}' has invalid value '{}': {}",
                    name, value, e
                ));
            }
        }

        Ok(())
    }

    /// Get all public signal names
    pub fn public_signal_names(&self) -> Vec<String> {
        self.signals
            .iter()
            .filter_map(|(name, signal)| {
                if signal.public {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all secret signal names
    pub fn secret_signal_names(&self) -> Vec<String> {
        self.signals
            .iter()
            .filter_map(|(name, signal)| {
                if !signal.public {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convert ProveRequest to Program format
    ///
    /// This is used internally to convert JSON API requests to the Program format
    /// which is then used by Circuit::from_program()
    pub fn to_program(&self) -> crate::api::Program {
        use crate::api::program::Signal as ProgramSignal;

        let mut secret = IndexMap::new();
        let mut public = IndexMap::new();

        for (name, signal) in &self.signals {
            let prog_signal = ProgramSignal {
                value: signal.value.clone(),
                encoding: signal.encoding,
            };

            if signal.public {
                public.insert(name.clone(), prog_signal);
            } else {
                secret.insert(name.clone(), prog_signal);
            }
        }

        crate::api::Program {
            version: PROOF_VERSION,
            secret,
            public,
            preprocess: self.preprocess.clone(),
            circuit: self.circuit.clone(),
        }
    }
}

impl VerifyRequest {
    /// Validate the verify request
    ///
    /// Checks that:
    /// - Proof is not empty
    /// - Verification context is not empty
    /// - All public signal values can be parsed according to their encoding
    pub fn validate(&self) -> Result<(), String> {
        if self.proof.is_empty() {
            return Err("Proof cannot be empty".to_string());
        }

        if self.verify_context.is_empty() {
            return Err("Verification context cannot be empty".to_string());
        }

        // Validate that all public signal values can be parsed
        for (name, signal) in &self.public_signals {
            let result = if let Some(encoding) = signal.encoding {
                // Use explicit encoding
                parse_value(&signal.value, encoding)
            } else {
                // Auto-detect encoding
                parse_value_auto(&signal.value)
            };

            if let Err(e) = result {
                return Err(format!(
                    "Public signal '{}' has invalid value '{}': {}",
                    name, signal.value, e
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_request_serialization() {
        let mut signals = IndexMap::new();
        signals.insert(
            "A".to_string(),
            Signal {
                value: Some("10".to_string()),
                encoding: None,
                public: false,
            },
        );
        signals.insert(
            "B".to_string(),
            Signal {
                value: Some("20".to_string()),
                encoding: None,
                public: true,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["(A + B) * C > D".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ProveRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.circuit, deserialized.circuit);
        assert_eq!(request.signals.len(), deserialized.signals.len());
    }

    #[test]
    fn test_signal_hex_encoding() {
        let signal = Signal {
            value: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string()),
            encoding: Some(ValueEncoding::Hex),
            public: true,
        };

        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: Signal = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.encoding, Some(ValueEncoding::Hex));
        assert_eq!(deserialized.value, signal.value);
    }

    #[test]
    fn test_signal_base58_encoding() {
        let signal = Signal {
            value: Some("9aE476sH92Vc7DMCzKNgWUiQ6UdC2DXf9v".to_string()),
            encoding: Some(ValueEncoding::Base58),
            public: false,
        };

        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: Signal = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.encoding, Some(ValueEncoding::Base58));
    }

    #[test]
    fn test_signal_base64_encoding() {
        let signal = Signal {
            value: Some("SGVsbG8gV29ybGQ=".to_string()),
            encoding: Some(ValueEncoding::Base64),
            public: true,
        };

        let json = serde_json::to_string(&signal).unwrap();
        assert!(json.contains("\"base64\""));
    }

    #[test]
    fn test_prove_request_validation() {
        let mut signals = IndexMap::new();
        signals.insert(
            "A".to_string(),
            Signal {
                value: Some("10".to_string()),
                encoding: None,
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["(A + B) > C".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_prove_request_invalid_value() {
        let mut signals = IndexMap::new();
        signals.insert(
            "A".to_string(),
            Signal {
                value: Some("not_a_number".to_string()),
                encoding: None,
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["A > B".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_public_secret_signal_separation() {
        let mut signals = IndexMap::new();
        signals.insert(
            "A".to_string(),
            Signal {
                value: Some("10".to_string()),
                encoding: None,
                public: false,
            },
        );
        signals.insert(
            "B".to_string(),
            Signal {
                value: Some("20".to_string()),
                encoding: None,
                public: true,
            },
        );
        signals.insert(
            "C".to_string(),
            Signal {
                value: Some("30".to_string()),
                encoding: None,
                public: true,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["(A + B) > C".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        let public_names = request.public_signal_names();
        let secret_names = request.secret_signal_names();

        assert_eq!(public_names.len(), 2);
        assert_eq!(secret_names.len(), 1);
        assert!(public_names.contains(&"B".to_string()));
        assert!(public_names.contains(&"C".to_string()));
        assert!(secret_names.contains(&"A".to_string()));
    }

    #[test]
    fn test_verify_response() {
        let response = VerifyResponse {
            valid: true,
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(!json.contains("error")); // Should be omitted when None
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse {
            error: "Proof verification failed".to_string(),
            details: Some("Invalid public inputs".to_string()),
        };

        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ErrorResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(error.error, deserialized.error);
        assert_eq!(error.details, deserialized.details);
    }

    #[test]
    fn test_validate_hex_encoding() {
        let mut signals = IndexMap::new();
        signals.insert(
            "addr".to_string(),
            Signal {
                // Fixed: Ethereum address must be 40 hex chars (20 bytes)
                value: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bE".to_string()),
                encoding: Some(ValueEncoding::Hex),
                public: true,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["addr > 0".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should pass validation (hex with explicit encoding)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_base58_encoding() {
        let mut signals = IndexMap::new();
        signals.insert(
            "solana_addr".to_string(),
            Signal {
                value: Some("9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM".to_string()),
                encoding: Some(ValueEncoding::Base58),
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["solana_addr == solana_addr".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should pass validation (base58 with explicit encoding)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_base64_encoding() {
        let mut signals = IndexMap::new();
        signals.insert(
            "data".to_string(),
            Signal {
                value: Some("SGVsbG8gV29ybGQ=".to_string()),
                encoding: Some(ValueEncoding::Base64),
                public: true,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["data > 0".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should pass validation (base64 with explicit encoding)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_auto_detect_hex() {
        let mut signals = IndexMap::new();
        signals.insert(
            "addr".to_string(),
            Signal {
                value: Some("0x1a2b".to_string()),
                encoding: None,  // Auto-detect
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["addr > 0".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should pass validation (hex auto-detected)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_hex() {
        let mut signals = IndexMap::new();
        signals.insert(
            "addr".to_string(),
            Signal {
                value: Some("0xZZZZ".to_string()),  // Invalid hex
                encoding: Some(ValueEncoding::Hex),
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["addr > 0".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should fail validation (invalid hex)
        let output = request.validate();
        assert!(output.is_err());
        assert!(output.unwrap_err().contains("addr"));
    }

    #[test]
    fn test_validate_invalid_base58() {
        let mut signals = IndexMap::new();
        signals.insert(
            "addr".to_string(),
            Signal {
                value: Some("0OIl".to_string()),  // Invalid base58 (contains 0, O, I, l)
                encoding: Some(ValueEncoding::Base58),
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["addr > 0".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should fail validation (invalid base58)
        let output = request.validate();
        assert!(output.is_err());
        assert!(output.unwrap_err().contains("addr"));
    }

    #[test]
    fn test_validate_large_decimal() {
        let mut signals = IndexMap::new();
        signals.insert(
            "large".to_string(),
            Signal {
                value: Some("999999999999999999999999999999".to_string()),  // Very large decimal
                encoding: None,
                public: false,
            },
        );

        let request = ProveRequest {
            preprocess: vec![],
            circuit: vec!["large > 0".to_string()],
            signals,
            strategy: Strategy::Auto,
        };

        // Should pass validation (large decimal is valid)
        assert!(request.validate().is_ok());
    }
}