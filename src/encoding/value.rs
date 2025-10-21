//! Value encoding and decoding utilities
//!
//! This module provides support for multiple value formats:
//! - Decimal strings: "12345" (arbitrary precision using BigUint)
//! - Hexadecimal: "0x1a2b" or "1a2b" (any size)
//! - Base58: "5HpH..." (Solana/Bitcoin addresses - 32 bytes)
//! - Base64: "SGVsbG8=" (universal encoding)
//!
//! # Important Notes
//!
//! - **Equality comparisons** (`==`, `!=`) work with values of any size
//! - **Ordering comparisons** (`>`, `<`, `>=`, `<=`) require values < 2^64 due to range proof constraints
//! - Use equality operators for comparing large values like Solana addresses (32 bytes)

use serde::{Deserialize, Serialize};
use thiserror::Error;
use num_bigint::BigUint;
use num_traits::Num;
use base64::{Engine as _, engine::general_purpose};

#[derive(Error, Debug)]
pub enum ValueEncodingError {
    #[error("Invalid decimal number: {0}")]
    InvalidDecimal(String),

    #[error("Invalid hexadecimal: {0}")]
    InvalidHex(String),

    #[error("Invalid base58: {0}")]
    InvalidBase58(String),

    #[error("Invalid base64: {0}")]
    InvalidBase64(String),

    #[error("Invalid base85: {0}")]
    InvalidBase85(String),

    #[error("Value too large (exceeds field size)")]
    ValueTooLarge,

    #[error("Unknown encoding format")]
    UnknownFormat,
}

/// Value encoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueEncoding {
    /// Decimal string: "12345"
    Decimal,

    /// Hexadecimal with or without 0x prefix: "0x1a2b" or "1a2b"
    Hex,

    /// Base58 encoding (Bitcoin/Solana): "5HpH..."
    Base58,

    /// Base64 encoding: "SGVsbG8="
    Base64,

    /// Base85 encoding (Ascii85): More compact than Base64
    Base85,

    /// Plain UTF-8 text: "hello" (for preprocessing inputs like hash functions)
    Text,
}

impl Default for ValueEncoding {
    fn default() -> Self {
        ValueEncoding::Decimal
    }
}

/// Parse a value string according to the specified encoding
///
/// # Examples
///
/// ```ignore
/// // Decimal
/// let val = parse_value("12345", ValueEncoding::Decimal)?;
///
/// // Hexadecimal (with or without 0x)
/// let val = parse_value("0x1a2b", ValueEncoding::Hex)?;
/// let val = parse_value("1a2b", ValueEncoding::Hex)?;
///
/// // Base58 (Solana pubkey)
/// let val = parse_value("9aE476sH92Vc7DMC...", ValueEncoding::Base58)?;
///
/// // Base64
/// let val = parse_value("SGVsbG8=", ValueEncoding::Base64)?;
/// ```
pub fn parse_value(value: &str, encoding: ValueEncoding) -> Result<Vec<u8>, ValueEncodingError> {
    match encoding {
        ValueEncoding::Decimal => parse_decimal(value),
        ValueEncoding::Hex => parse_hex(value),
        ValueEncoding::Base58 => parse_base58(value),
        ValueEncoding::Base64 => parse_base64(value),
        ValueEncoding::Base85 => parse_base85(value),
        ValueEncoding::Text => Ok(value.as_bytes().to_vec()),
    }
}

/// Auto-detect encoding format and parse value
///
/// Detection rules:
/// - Starts with "0x" -> Hex
/// - All digits -> Decimal
/// - Contains only base58 chars -> Base58
/// - Contains base64 chars (including +/=) -> Base64
/// - Everything else -> Text (UTF-8 string)
pub fn parse_value_auto(value: &str) -> Result<Vec<u8>, ValueEncodingError> {
    // Try hex first (most specific)
    if value.starts_with("0x") || value.starts_with("0X") {
        return parse_hex(value);
    }

    // Try decimal (simple and common)
    if value.chars().all(|c| c.is_ascii_digit()) {
        return parse_decimal(value);
    }

    // Try base64 (contains +, /, =)
    if value.contains('+') || value.contains('/') || value.contains('=') {
        if let Ok(result) = parse_base64(value) {
            return Ok(result);
        }
    }

    // Try base58 (no 0, O, I, l characters)
    if value.chars().all(|c| {
        c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l'
    }) {
        if let Ok(result) = parse_base58(value) {
            return Ok(result);
        }
    }

    // Default to plain text (UTF-8 bytes)
    // This allows arbitrary strings to be used in preprocessing
    Ok(value.as_bytes().to_vec())
}

fn parse_decimal(value: &str) -> Result<Vec<u8>, ValueEncodingError> {
    // Empty string is invalid
    if value.is_empty() {
        return Err(ValueEncodingError::InvalidDecimal("empty string".to_string()));
    }

    // Parse as BigUint (supports arbitrary precision)
    // This correctly handles any decimal number, including very large ones
    let num = BigUint::from_str_radix(value, 10)
        .map_err(|_| ValueEncodingError::InvalidDecimal(value.to_string()))?;

    // Convert to big-endian bytes
    let bytes = num.to_bytes_be();

    // Return at least 1 byte (even for 0)
    if bytes.is_empty() {
        Ok(vec![0])
    } else {
        Ok(bytes)
    }
}

fn parse_hex(value: &str) -> Result<Vec<u8>, ValueEncodingError> {
    // Remove 0x prefix if present
    let hex_str = value.strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);

    hex::decode(hex_str)
        .map_err(|_| ValueEncodingError::InvalidHex(value.to_string()))
}

fn parse_base58(value: &str) -> Result<Vec<u8>, ValueEncodingError> {
    bs58::decode(value)
        .into_vec()
        .map_err(|_| ValueEncodingError::InvalidBase58(value.to_string()))
}

fn parse_base64(value: &str) -> Result<Vec<u8>, ValueEncodingError> {
    general_purpose::STANDARD.decode(value)
        .map_err(|_| ValueEncodingError::InvalidBase64(value.to_string()))
}

fn parse_base85(value: &str) -> Result<Vec<u8>, ValueEncodingError> {
    ascii85::decode(value)
        .map_err(|_| ValueEncodingError::InvalidBase85(value.to_string()))
}

/// Convert bytes to decimal string representation
pub fn bytes_to_decimal(bytes: &[u8]) -> String {
    // Use BigUint for arbitrary precision
    // Works correctly for any size
    BigUint::from_bytes_be(bytes).to_string()
}

/// Convert bytes to hex string (with 0x prefix)
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Convert bytes to base58 string
pub fn bytes_to_base58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

/// Convert bytes to base64 string
pub fn bytes_to_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

/// Convert bytes to base85 string (Adobe ASCII85)
pub fn bytes_to_base85(bytes: &[u8]) -> String {
    ascii85::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        let result = parse_value("12345", ValueEncoding::Decimal).unwrap();
        assert_eq!(bytes_to_decimal(&result), "12345");
    }

    #[test]
    fn test_parse_hex_with_prefix() {
        let result = parse_value("0x1a2b", ValueEncoding::Hex).unwrap();
        assert_eq!(result, vec![0x1a, 0x2b]);
    }

    #[test]
    fn test_parse_hex_without_prefix() {
        let result = parse_value("1a2b", ValueEncoding::Hex).unwrap();
        assert_eq!(result, vec![0x1a, 0x2b]);
    }

    #[test]
    fn test_parse_base64() {
        let original = b"Hello, World!";
        let encoded = bytes_to_base64(original);
        let decoded = parse_value(&encoded, ValueEncoding::Base64).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_parse_base58() {
        let original = b"Hello, World!";
        let encoded = bs58::encode(original).into_string();
        let decoded = parse_value(&encoded, ValueEncoding::Base58).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_auto_detect_hex() {
        let result = parse_value_auto("0x1a2b").unwrap();
        assert_eq!(result, vec![0x1a, 0x2b]);
    }

    #[test]
    fn test_auto_detect_decimal() {
        let result = parse_value_auto("12345").unwrap();
        assert_eq!(bytes_to_decimal(&result), "12345");
    }

    #[test]
    fn test_roundtrip_hex() {
        let original = vec![0xde, 0xad, 0xbe, 0xef];
        let hex_str = bytes_to_hex(&original);
        let decoded = parse_value(&hex_str, ValueEncoding::Hex).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_base58() {
        let original = b"Solana Public Key";
        let encoded = bytes_to_base58(original);
        let decoded = parse_value(&encoded, ValueEncoding::Base58).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_large_decimal_numbers() {
        // Test with number larger than u64::MAX (18446744073709551615)
        let large_number = "99999999999999999999999999999999";
        let bytes = parse_value(large_number, ValueEncoding::Decimal).unwrap();
        let recovered = bytes_to_decimal(&bytes);
        assert_eq!(recovered, large_number);
    }

    #[test]
    fn test_solana_address_as_decimal() {
        // Real Solana address: 32 bytes
        let address_base58 = "9aE476sH92Vc7DMC8bZNpe1xNNNy1fNjFpCGvfMuZMwM";

        // Decode from base58
        let bytes = parse_value(address_base58, ValueEncoding::Base58).unwrap();
        assert_eq!(bytes.len(), 32);

        // Convert to decimal representation
        let as_decimal = bytes_to_decimal(&bytes);

        // Convert back from decimal
        let recovered_bytes = parse_value(&as_decimal, ValueEncoding::Decimal).unwrap();

        // Should match original bytes
        assert_eq!(recovered_bytes, bytes);
    }

    #[test]
    fn test_auto_detect_text() {
        // Plain text strings should be parsed as UTF-8 bytes
        let result = parse_value_auto("Hello").unwrap();
        assert_eq!(result, b"Hello");

        // Test with more complex text
        let result2 = parse_value_auto("Hello, World!").unwrap();
        assert_eq!(result2, b"Hello, World!");
    }

    #[test]
    fn test_consistency_minimal_bytes() {
        // Both decimal and hex should return minimal byte representation
        let decimal_10 = parse_value("10", ValueEncoding::Decimal).unwrap();
        let hex_0a = parse_value("0x0a", ValueEncoding::Hex).unwrap();

        // Both should be [10] (1 byte), not [0, 0, 0, 0, 0, 0, 0, 10] (8 bytes)
        assert_eq!(decimal_10, vec![10]);
        assert_eq!(hex_0a, vec![10]);
        assert_eq!(decimal_10, hex_0a);
    }

    #[test]
    fn test_zero_representation() {
        // Zero should be represented as [0] (1 byte)
        let decimal_0 = parse_value("0", ValueEncoding::Decimal).unwrap();
        let hex_0 = parse_value("0x00", ValueEncoding::Hex).unwrap();

        assert_eq!(decimal_0, vec![0]);
        assert_eq!(hex_0, vec![0]);
    }

    #[test]
    fn test_parse_base85() {
        let original = b"Hello, World!";
        let encoded = bytes_to_base85(original);
        let decoded = parse_value(&encoded, ValueEncoding::Base85).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_base85() {
        let original = b"Test data for Base85";
        let encoded = bytes_to_base85(original);
        let decoded = parse_value(&encoded, ValueEncoding::Base85).unwrap();
        assert_eq!(decoded, original);
    }
}