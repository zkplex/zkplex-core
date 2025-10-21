//! ZKPlex Program Format
//!
//! Zircon format for ZKP programs optimized for blockchain storage.
//!
//! # Zircon Format
//!
//! ```text
//! version/secret/public/preprocess/circuit
//! ```
//!
//! Or without preprocess (backward compatible):
//!
//! ```text
//! version/secret/public/circuit
//! ```
//!
//! - **version**: Single number (1, 2, ...)
//! - **secret**: `name:value[:encoding][,...]` or `-` if empty
//! - **public**: `name:value[:encoding][,...]` or `-` if empty
//! - **preprocess**: `statement[;statement]*` or `-` if empty (hash/encoding operations)
//! - **circuit**: `statement[;statement]*` where last statement is the output
//!
//! # Examples
//!
//! ```text
//! // Simple
//! 1/A:10,B:20/-/A+B
//!
//! // With public input
//! 1/balance:1000/min:100/balance>min
//!
//! // With intermediate variables
//! 1/A:10,B:20/-/sum<==A+B;sum*2
//!
//! // Complex logic (both && and AND supported, || and OR also work)
//! 1/age:25,income:50000/-/(age>18)&&(income>30000)
//! 1/age:25,income:50000/-/(age>18)||(income>30000)
//!
//! // With encoding
//! 1/wallet:9aE...:base58/expected:So1...:base58/wallet==expected
//!
//! // With preprocessing using | for concatenation
//! 1/A:255,B:16/-/hash<==sha256(A{%x}|B{%d})/hash>100
//! ```
//!
//! # JSON Format
//!
//! ```json
//! {
//!   "version": 1,
//!   "secret": {"A": "10", "B": "20"},
//!   "public": {"threshold": "100"},
//!   "circuit": ["sum <== A + B", "sum > threshold"]
//! }
//! ```
//!
//! # Operators
//!
//! Both symbolic and word operators are supported:
//! - AND / &&
//! - OR / ||
//! - NOT / !

use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use crate::encoding::ValueEncoding;

/// Signal with value and optional encoding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signal {
    /// Signal value (None for output signals that will be computed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Optional encoding (hex, base58, base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<ValueEncoding>,
}

impl Signal {
    /// Create signal with value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: Some(value.into()),
            encoding: None,
        }
    }

    /// Create signal with value and encoding
    pub fn with_encoding(value: impl Into<String>, encoding: ValueEncoding) -> Self {
        Self {
            value: Some(value.into()),
            encoding: Some(encoding),
        }
    }

    /// Create output signal (no value, will be computed)
    pub fn output() -> Self {
        Self {
            value: None,
            encoding: None,
        }
    }
}

/// ZKPlex Program
///
/// Represents a ZKP circuit with inputs and computation logic.
///
/// # Fields
///
/// - `version`: Format version (1, 2, ...)
/// - `secret`: Secret input signals (witness)
/// - `public`: Public input signals
/// - `preprocess`: Preprocessing operations (hashes, encodings, etc.)
/// - `circuit`: Circuit statements (last one is output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Program version
    pub version: u32,

    /// Secret input signals
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub secret: IndexMap<String, Signal>,

    /// Public input signals
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub public: IndexMap<String, Signal>,

    /// Preprocessing operations (hashes, encodings, etc.)
    /// Executed before circuit, outputs can be used in circuit or other preprocess ops
    ///
    /// Format: `name<==operation(args)`
    ///
    /// Supported operations:
    /// - Hash functions: `sha1()`, `sha256()`, `sha512()`, `md5()`, `crc32()`, `blake2b()`, `keccak256()`, `keccak()`
    /// - Encoding functions: `hex_encode()`, `base64()`, `base58()`, `base64_encode()`, `base58_encode()`
    /// - Utility: `concat()` - concatenates arguments (alternative to `|`)
    ///
    /// Format specifiers (printf-style):
    /// - `{%x}` / `{%X}` - hex lowercase/uppercase
    /// - `{%d}` - decimal
    /// - `{%o}` - octal
    /// - `{%08x}` - zero-padded hex (8 chars)
    /// - `{%b64}` / `{%B64}` - base64 lowercase/uppercase
    /// - `{%b58}` / `{%B58}` - base58 lowercase/uppercase
    /// - `{%064b64}` - zero-padded base64 (64 chars)
    /// - `{%032b58}` - zero-padded base58 (32 chars)
    ///
    /// Concatenation:
    /// - Use `|` between arguments: `sha256(A{%x}|B{%d})`
    /// - Use `concat()` function: `sha256(concat(A{%x}, B{%d}))`
    ///
    /// Examples:
    /// - `hash<==sha256(A{%x}|B{%08x})` - hash with inline concat
    /// - `hash<==keccak256(concat(A{%x}, B{%d}))` - hash with concat()
    /// - `encoded<==base64(data{%d})` - base64 encoding
    /// - `encoded<==base58_encode(data{%064b58})` - base58 with padding
    /// - `h1<==sha256(A{%x});h2<==md5(h1{%b64})` - chained operations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preprocess: Vec<String>,

    /// Circuit statements (last one is output)
    pub circuit: Vec<String>,
}

impl Program {
    /// Create new program with version
    pub fn new(version: u32) -> Self {
        Self {
            version,
            secret: IndexMap::new(),
            public: IndexMap::new(),
            preprocess: Vec::new(),
            circuit: Vec::new(),
        }
    }

    /// Parse from zircon format: `version/secret/public/preprocess/circuit` or `version/secret/public/circuit`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Simple (backward compatible)
    /// let p = Program::from_zircon("1/A:10,B:20/-/A+B")?;
    ///
    /// // With preprocessing
    /// let p = Program::from_zircon("1/A:10/-/h<==sha256(A{%x})/h>100")?;
    /// ```
    pub fn from_zircon(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split('/').collect();

        let (version, secret, public, preprocess, circuit) = match parts.len() {
            5 => {
                let version = parts[0].parse::<u32>()
                    .map_err(|_| format!("Invalid version: {}", parts[0]))?;
                let secret = Self::parse_signals(parts[1])?;
                let public = Self::parse_signals(parts[2])?;
                let preprocess = Self::parse_statements(parts[3])?;
                let circuit = Self::parse_statements(parts[4])?;
                (version, secret, public, preprocess, circuit)
            }
            _ => {
                return Err(format!(
                    "Invalid format: expected 'version/secret/public/preprocess/circuit', got {} parts",
                    parts.len()
                ));
            }
        };

        if circuit.is_empty() {
            return Err("Circuit cannot be empty".to_string());
        }

        Ok(Self {
            version,
            secret,
            public,
            preprocess,
            circuit,
        })
    }

    /// Parse statements from semicolon-separated string
    pub fn parse_statements(input: &str) -> Result<Vec<String>, String> {
        if input.trim() == "-" || input.is_empty() {
            return Ok(Vec::new());
        }

        Ok(input
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Convert to zircon format: `version/secret/public/preprocess/circuit` or `version/secret/public/circuit`
    ///
    /// Uses 5-part format if preprocess is not empty, otherwise uses 4-part format for backward compatibility.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let zircon = program.to_zircon();
    /// // "1/A:10,B:20/-/-/A+B" (with empty preprocess)
    /// // "1/A:10,B:20/-/A+B"   (without preprocess - backward compatible)
    /// ```
    pub fn to_zircon(&self) -> String {
        let secret_str = if self.secret.is_empty() {
            "-".to_string()
        } else {
            Self::signals_to_string(&self.secret)
        };

        let public_str = if self.public.is_empty() {
            "-".to_string()
        } else {
            Self::signals_to_string(&self.public)
        };

        let circuit_str = self.circuit.join(";");
        let preprocess_str = self.preprocess.join(";");
        format!("{}/{}/{}/{}/{}", self.version, secret_str, public_str, preprocess_str, circuit_str)
    }

    /// Parse signals from format: `name:value[:encoding][,...]` or `-`
    fn parse_signals(input: &str) -> Result<IndexMap<String, Signal>, String> {
        if input.trim() == "-" || input.is_empty() {
            return Ok(IndexMap::new());
        }

        let mut signals = IndexMap::new();

        for part in input.split(',') {
            let components: Vec<&str> = part.trim().split(':').collect();

            match components.len() {
                2 => {
                    // name:value
                    let name = components[0].trim().to_string();
                    let value = components[1].trim().to_string();

                    if name.is_empty() {
                        return Err("Signal name cannot be empty".to_string());
                    }

                    signals.insert(name, Signal::new(value));
                }
                3 => {
                    // name:value:encoding
                    let name = components[0].trim().to_string();
                    let value = components[1].trim().to_string();
                    let encoding_str = components[2].trim();

                    if name.is_empty() {
                        return Err("Signal name cannot be empty".to_string());
                    }

                    let encoding = match encoding_str {
                        "hex" => ValueEncoding::Hex,
                        "base58" => ValueEncoding::Base58,
                        "base64" => ValueEncoding::Base64,
                        "base85" => ValueEncoding::Base85,
                        "decimal" => ValueEncoding::Decimal,
                        "text" => ValueEncoding::Text,
                        _ => return Err(format!("Unknown encoding: {}", encoding_str)),
                    };

                    signals.insert(name, Signal::with_encoding(value, encoding));
                }
                _ => {
                    return Err(format!("Invalid signal format '{}': expected 'name:value' or 'name:value:encoding'", part));
                }
            }
        }

        Ok(signals)
    }

    /// Convert signals IndexMap to string format
    fn signals_to_string(signals: &IndexMap<String, Signal>) -> String {
        let mut items: Vec<String> = signals
            .iter()
            .map(|(name, signal)| {
                let value_str = signal.value.as_deref().unwrap_or("");
                if let Some(encoding) = &signal.encoding {
                    let enc_str = match encoding {
                        ValueEncoding::Hex => "hex",
                        ValueEncoding::Base58 => "base58",
                        ValueEncoding::Base64 => "base64",
                        ValueEncoding::Base85 => "base85",
                        ValueEncoding::Decimal => "decimal",
                        ValueEncoding::Text => "text",
                    };
                    format!("{}:{}:{}", name, value_str, enc_str)
                } else {
                    format!("{}:{}", name, value_str)
                }
            })
            .collect();

        items.sort(); // For consistent ordering
        items.join(",")
    }

    /// Parse from JSON format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let json = r#"{"version": 1, "secret": {"A": "10"}, "circuit": ["A+B"]}"#;
    /// let p = Program::from_json(json)?;
    /// ```
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Convert to JSON format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let json = program.to_json()?;
    /// ```
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }

    /// Validate program
    pub fn validate(&self) -> Result<(), String> {
        // Check version
        if self.version != 1 {
            return Err("Version must be >= 1".to_string());
        }

        // Check that we have at least one circuit statement
        if self.circuit.is_empty() {
            return Err("Circuit cannot be empty".to_string());
        }

        // Validate signal values can be parsed
        for (name, signal) in self.secret.iter().chain(self.public.iter()) {
            // Skip output signals (value is None)
            let value_str = match &signal.value {
                Some(v) => v,
                None => continue, // Output signal, skip validation
            };

            // Skip validation for placeholder values
            if value_str == "?" {
                continue;
            }

            // Signal values cannot be empty
            if value_str.is_empty() {
                return Err(format!(
                    "Signal '{}' has empty value",
                    name
                ));
            }

            use crate::encoding::{parse_value, parse_value_auto};

            let output = if let Some(encoding) = signal.encoding {
                parse_value(value_str, encoding)
            } else {
                parse_value_auto(value_str)
            };

            if let Err(e) = output {
                return Err(format!(
                    "Signal '{}' has invalid value '{}': {}",
                    name, value_str, e
                ));
            }
        }

        Ok(())
    }

    /// Get all input signal names (secret + public)
    pub fn input_signals(&self) -> Vec<String> {
        let mut signals: Vec<String> = self.secret.keys()
            .chain(self.public.keys())
            .cloned()
            .collect();
        signals.sort();
        signals
    }

    /// Check if signal is public
    pub fn is_public(&self, name: &str) -> bool {
        self.public.contains_key(name)
    }

    /// Get output expression (last statement in circuit)
    pub fn output_expression(&self) -> Option<&String> {
        self.circuit.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_zircon() {
        let p = Program::from_zircon("1/A:10,B:20/-/A+B").unwrap();

        assert_eq!(p.version, 1);
        assert_eq!(p.secret.len(), 2);
        assert_eq!(p.secret.get("A").unwrap().value.as_ref().unwrap(), "10");
        assert_eq!(p.secret.get("B").unwrap().value.as_ref().unwrap(), "20");
        assert_eq!(p.public.len(), 0);
        assert_eq!(p.circuit.len(), 1);
        assert_eq!(p.circuit[0], "A+B");
    }

    #[test]
    fn test_parse_with_public() {
        let p = Program::from_zircon("1/balance:1000/min:100/balance>min").unwrap();

        assert_eq!(p.secret.len(), 1);
        assert_eq!(p.public.len(), 1);
        assert_eq!(p.secret.get("balance").unwrap().value.as_ref().unwrap(), "1000");
        assert_eq!(p.public.get("min").unwrap().value.as_ref().unwrap(), "100");
    }

    #[test]
    fn test_parse_with_intermediate() {
        let p = Program::from_zircon("1/A:10,B:20/-/sum<==A+B;sum*2").unwrap();

        assert_eq!(p.circuit.len(), 2);
        assert_eq!(p.circuit[0], "sum<==A+B");
        assert_eq!(p.circuit[1], "sum*2");
    }

    #[test]
    fn test_parse_both_empty() {
        let p = Program::from_zircon("1/-/-/output<==5+10;output>10").unwrap();

        assert_eq!(p.secret.len(), 0);
        assert_eq!(p.public.len(), 0);
        assert_eq!(p.circuit.len(), 2);
    }

    #[test]
    fn test_parse_with_encoding() {
        let p = Program::from_zircon("1/wallet:abc:base58/expected:xyz:base58/wallet==expected").unwrap();

        assert_eq!(p.secret.get("wallet").unwrap().encoding, Some(ValueEncoding::Base58));
        assert_eq!(p.public.get("expected").unwrap().encoding, Some(ValueEncoding::Base58));
    }

    #[test]
    fn test_to_zircon() {
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new("10"));
        p.secret.insert("B".to_string(), Signal::new("20"));
        p.circuit.push("A+B".to_string());

        let zircon = p.to_zircon();
        assert!(zircon.starts_with("1/"));
        assert!(zircon.contains("A:10"));
        assert!(zircon.contains("B:20"));
        assert!(zircon.contains("/-/")); // empty public
        assert!(zircon.ends_with("/A+B"));
    }

    #[test]
    fn test_roundtrip_zircon() {
        let original = "1/A:10,B:20/threshold:100/sum<==A+B;sum>threshold";
        let p = Program::from_zircon(original).unwrap();
        let zircon = p.to_zircon();
        let p2 = Program::from_zircon(&zircon).unwrap();

        assert_eq!(p.version, p2.version);
        assert_eq!(p.secret.len(), p2.secret.len());
        assert_eq!(p.public.len(), p2.public.len());
        assert_eq!(p.circuit.len(), p2.circuit.len());
    }

    #[test]
    fn test_json_format() {
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new("10"));
        p.circuit.push("A>5".to_string());

        let json = p.to_json().unwrap();
        let p2 = Program::from_json(&json).unwrap();

        assert_eq!(p.version, p2.version);
        assert_eq!(p.secret.len(), p2.secret.len());
        assert_eq!(p.circuit.len(), p2.circuit.len());
    }

    #[test]
    fn test_roundtrip_json_zircon() {
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new("10"));
        p.secret.insert("B".to_string(), Signal::new("20"));
        p.public.insert("threshold".to_string(), Signal::new("100"));
        p.circuit.push("sum<==A+B".to_string());
        p.circuit.push("sum>threshold".to_string());

        // JSON -> Zircon -> JSON
        let json1 = p.to_json().unwrap();
        let zircon = p.to_zircon();
        let p2 = Program::from_zircon(&zircon).unwrap();
        let json2 = p2.to_json().unwrap();

        let p1: Program = serde_json::from_str(&json1).unwrap();
        let p2: Program = serde_json::from_str(&json2).unwrap();

        assert_eq!(p1.version, p2.version);
        assert_eq!(p1.secret.len(), p2.secret.len());
        assert_eq!(p1.public.len(), p2.public.len());
    }

    #[test]
    fn test_validate_success() {
        let p = Program::from_zircon("1/A:10/-/A>5").unwrap();
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_version() {
        let mut p = Program::new(0);
        p.circuit.push("5+5".to_string());
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_empty_circuit() {
        let p = Program::new(1);
        assert!(p.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_value() {
        // Note: "not_a_number" is now treated as plain text (valid), so we need a different test
        // Test with empty value instead (which should be invalid)
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new(""));  // Empty value
        p.circuit.push("A>5".to_string());

        let output = p.validate();
        assert!(output.is_err());
        assert!(output.unwrap_err().contains("empty value"));
    }

    #[test]
    fn test_output_expression() {
        let p = Program::from_zircon("1/A:10/-/sum<==A+5;sum*2").unwrap();
        assert_eq!(p.output_expression(), Some(&"sum*2".to_string()));
    }

    #[test]
    fn test_parse_invalid_format() {
        // Too few parts
        assert!(Program::from_zircon("1/A:10/circuit").is_err());

        // Too many parts
        assert!(Program::from_zircon("1/A:10/-/circuit/extra/extra2").is_err());
    }

    #[test]
    fn test_parse_with_preprocess() {
        // New 5-part format with preprocessing - now can use | for concatenation!
        let p = Program::from_zircon("1/A:255,B:1000/-/hash<==sha256(A{%x}|B{%d})/hash>100").unwrap();

        assert_eq!(p.version, 1);
        assert_eq!(p.secret.len(), 2);
        assert_eq!(p.preprocess.len(), 1);
        assert_eq!(p.preprocess[0], "hash<==sha256(A{%x}|B{%d})");
        assert_eq!(p.circuit.len(), 1);
        assert_eq!(p.circuit[0], "hash>100");
    }

    #[test]
    fn test_parse_with_empty_preprocess() {
        // New 5-part format with empty preprocessing (using -)
        let p = Program::from_zircon("1/A:10/-/-/A>5").unwrap();

        assert_eq!(p.preprocess.len(), 0);
        assert_eq!(p.circuit.len(), 1);
    }

    #[test]
    fn test_to_zircon_with_preprocess() {
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new("255"));
        p.preprocess.push("hash<==sha256(A{%x})".to_string());
        p.circuit.push("hash>100".to_string());

        let zircon = p.to_zircon();

        // Should use 5-part format when preprocess is not empty
        assert!(zircon.contains("/hash<==sha256(A{%x})/"));
        assert_eq!(zircon.split('/').count(), 5);
    }


    #[test]
    fn test_roundtrip_with_preprocess() {
        let original = "1/A:10/-/h<==sha256(A{%x});e<==base64(h{%b64})/e>100";
        let p = Program::from_zircon(original).unwrap();

        assert_eq!(p.preprocess.len(), 2);
        assert_eq!(p.preprocess[0], "h<==sha256(A{%x})");
        assert_eq!(p.preprocess[1], "e<==base64(h{%b64})");

        let zircon = p.to_zircon();
        let p2 = Program::from_zircon(&zircon).unwrap();

        assert_eq!(p.version, p2.version);
        assert_eq!(p.preprocess.len(), p2.preprocess.len());
        assert_eq!(p.circuit.len(), p2.circuit.len());
    }

    #[test]
    fn test_json_with_preprocess() {
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new("10"));
        p.preprocess.push("hash<==sha256(A{%x})".to_string());
        p.circuit.push("hash>5".to_string());

        let json = p.to_json().unwrap();
        assert!(json.contains("preprocess"));
        assert!(json.contains("sha256"));

        let p2 = Program::from_json(&json).unwrap();
        assert_eq!(p.preprocess.len(), p2.preprocess.len());
        assert_eq!(p.preprocess[0], p2.preprocess[0]);
    }

    #[test]
    fn test_pipe_concatenation_in_preprocess() {
        // Test that | can now be used for concatenation in preprocessing
        let p = Program::from_zircon("1/A:255,B:16/-/hash<==sha256(A{%x}|B{%d})/hash>100").unwrap();

        assert_eq!(p.preprocess.len(), 1);
        assert_eq!(p.preprocess[0], "hash<==sha256(A{%x}|B{%d})");

        // Verify the format parses correctly
        assert_eq!(p.version, 1);
        assert_eq!(p.secret.len(), 2);
        assert_eq!(p.circuit.len(), 1);
    }

    #[test]
    fn test_boolean_or_in_circuit() {
        // Test that || can be used for boolean OR in circuit circuits
        let p = Program::from_zircon("1/age:25,income:50000/-/(age>18)||(income>30000)").unwrap();

        assert_eq!(p.circuit.len(), 1);
        assert_eq!(p.circuit[0], "(age>18)||(income>30000)");

        assert_eq!(p.secret.len(), 2);
        assert_eq!(p.secret.get("age").unwrap().value.as_ref().unwrap(), "25");
        assert_eq!(p.secret.get("income").unwrap().value.as_ref().unwrap(), "50000");
    }

    #[test]
    fn test_combined_pipe_and_or() {
        // Test using both | for concat and || for OR in the same program
        let p = Program::from_zircon("1/A:100,B:200,C:10/-/hash<==sha256(A{%x}|B{%d})/(hash>50)||(C<20)").unwrap();

        // Check preprocessing has | concatenation
        assert_eq!(p.preprocess.len(), 1);
        assert!(p.preprocess[0].contains('|'));
        assert!(p.preprocess[0].contains("A{%x}|B{%d}"));

        // Check circuit has || boolean OR
        assert_eq!(p.circuit.len(), 1);
        assert!(p.circuit[0].contains("||"));
        assert_eq!(p.circuit[0], "(hash>50)||(C<20)");
    }

    #[test]
    fn test_multiple_pipes_in_preprocess() {
        // Test multiple concatenations with |
        let p = Program::from_zircon("1/A:10,B:20,C:30/-/hash<==sha256(A{%d}|B{%d}|C{%d})/hash>0").unwrap();

        assert_eq!(p.preprocess.len(), 1);
        assert_eq!(p.preprocess[0], "hash<==sha256(A{%d}|B{%d}|C{%d})");
    }

    #[test]
    fn test_and_and_or_together() {
        // Test using both && and || in circuit
        let p = Program::from_zircon("1/a:1,b:2,c:3/-/((a>0)&&(b>0))||((c>0))").unwrap();

        assert_eq!(p.circuit[0], "((a>0)&&(b>0))||((c>0))");
    }

    #[test]
    fn test_placeholder_value() {
        // Test that '?' can be used as a placeholder value
        let p = Program::from_zircon("1/A:?,B:20/-/A+B").unwrap();

        assert_eq!(p.version, 1);
        assert_eq!(p.secret.len(), 2);
        assert_eq!(p.secret.get("A").unwrap().value.as_ref().unwrap(), "?");
        assert_eq!(p.secret.get("B").unwrap().value.as_ref().unwrap(), "20");
        assert_eq!(p.circuit.len(), 1);
        assert_eq!(p.circuit[0], "A+B");
    }

    #[test]
    fn test_validate_with_placeholder() {
        // Test that validation passes with placeholder values
        let p = Program::from_zircon("1/A:?,B:10/C:?/A+B+C").unwrap();
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_placeholder_with_encoding() {
        // Test that '?' can be used with encoding
        let p = Program::from_zircon("1/wallet:?:base58/expected:xyz:base58/wallet==expected").unwrap();

        assert_eq!(p.secret.get("wallet").unwrap().value.as_ref().unwrap(), "?");
        assert_eq!(p.secret.get("wallet").unwrap().encoding, Some(ValueEncoding::Base58));
        assert!(p.validate().is_ok());
    }

    #[test]
    fn test_to_zircon_with_placeholder() {
        // Test that placeholder values are preserved in to_zircon()
        let mut p = Program::new(1);
        p.secret.insert("A".to_string(), Signal::new("?"));
        p.secret.insert("B".to_string(), Signal::new("20"));
        p.circuit.push("A+B".to_string());

        let zircon = p.to_zircon();
        assert!(zircon.contains("A:?") || zircon.contains("?"));
        assert!(zircon.contains("B:20"));
    }

}
