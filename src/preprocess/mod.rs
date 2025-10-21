//! Preprocessing operations for ZKP programs
//!
//! This module provides hash functions, encoding operations, and printf-style formatting
//! for preprocessing signals before circuit execution.
//!
//! # Supported Operations
//!
//! ## Hash Functions
//! - SHA-1, SHA-256, SHA-512
//! - MD5, CRC32
//! - BLAKE2b
//! - Keccak-256 (Ethereum)
//!
//! ## Encoding Functions
//! - hex_encode, base64_encode, base58_encode
//! - concat (string concatenation)
//!
//! ## Format Specifiers (printf-style)
//! - `{%x}` / `{%X}` - hex lowercase/uppercase
//! - `{%d}` - decimal
//! - `{%o}` - octal
//! - `{%08x}` - zero-padded hex (8 chars)
//! - `{%b64}` / `{%B64}` - base64 lowercase/uppercase
//! - `{%b58}` / `{%B58}` - base58 lowercase/uppercase
//! - `{%064b64}` - zero-padded base64 (64 chars)
//! - `{%032b58}` - zero-padded base58 (32 chars)

mod formatter;
mod hasher;

pub use formatter::format_value;
pub use hasher::{hash, HashAlgorithm};

use std::collections::HashMap;

/// Execute preprocessing operations on signals
///
/// Takes preprocess statements and signal values, executes operations in order,
/// and returns resulting intermediate signals.
///
/// # Arguments
///
/// * `statements` - Preprocessing statements (e.g., "hash<==sha256(A{%x})")
/// * `signals` - Input signal values
///
/// # Returns
///
/// HashMap of intermediate signal names and their computed values (as bytes)
///
/// # Example
///
/// ```ignore
/// let statements = vec!["hash<==sha256(A{%x})".to_string()];
/// let mut signals = HashMap::new();
/// signals.insert("A".to_string(), vec![255]);
///
/// let outputs = execute_preprocess(&statements, &signals)?;
/// // outputs["hash"] contains SHA-256 hash of "ff"
/// ```
pub fn execute_preprocess(
    statements: &[String],
    signals: &HashMap<String, Vec<u8>>,
) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut outputs = HashMap::new();

    // Execute each statement in order
    for statement in statements {
        let (name, value) = execute_statement(statement, signals, &outputs)?;
        outputs.insert(name, value);
    }

    Ok(outputs)
}

/// Execute a single preprocessing statement
///
/// # Format
///
/// `name<==operation(args)`
///
/// # Examples
///
/// - `hash<==sha256(A{%x})`
/// - `encoded<==base64(data{%d})`
/// - `combined<==concat(A{%x}, B{%d})`
fn execute_statement(
    statement: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<(String, Vec<u8>), String> {
    // Parse assignment: name<==operation(args)
    let parts: Vec<&str> = statement.split("<==").collect();
    if parts.len() != 2 {
        return Err(format!("Invalid preprocess statement: {}", statement));
    }

    let name = parts[0].trim().to_string();
    let operation = parts[1].trim();

    // Parse operation: function_name(args)
    if let Some(open_paren) = operation.find('(') {
        if !operation.ends_with(')') {
            return Err(format!("Missing closing parenthesis: {}", operation));
        }

        let func_name = operation[..open_paren].trim();
        let args_str = &operation[open_paren + 1..operation.len() - 1];

        // Execute operation
        let output = match func_name {
            // Hash functions
            "sha1" => execute_hash(HashAlgorithm::SHA1, args_str, input_signals, intermediate_signals)?,
            "sha256" => execute_hash(HashAlgorithm::SHA256, args_str, input_signals, intermediate_signals)?,
            "sha512" => execute_hash(HashAlgorithm::SHA512, args_str, input_signals, intermediate_signals)?,
            "md5" => execute_hash(HashAlgorithm::MD5, args_str, input_signals, intermediate_signals)?,
            "blake2b" => execute_hash(HashAlgorithm::BLAKE2b, args_str, input_signals, intermediate_signals)?,
            "keccak256" | "keccak" => execute_hash(HashAlgorithm::Keccak256, args_str, input_signals, intermediate_signals)?,
            "crc32" => execute_hash(HashAlgorithm::CRC32, args_str, input_signals, intermediate_signals)?,

            // Encoding functions
            "hex_encode" => execute_hex_encode(args_str, input_signals, intermediate_signals)?,
            "base64" | "base64_encode" => execute_base64_encode(args_str, input_signals, intermediate_signals)?,
            "base58" | "base58_encode" => execute_base58_encode(args_str, input_signals, intermediate_signals)?,

            // Utility
            "concat" => execute_concat(args_str, input_signals, intermediate_signals)?,

            _ => return Err(format!("Unknown function: {}", func_name)),
        };

        Ok((name, output))
    } else {
        Err(format!("Invalid operation format: {}", operation))
    }
}

/// Execute hash function on formatted arguments
fn execute_hash(
    algorithm: HashAlgorithm,
    args: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    // Parse and format arguments (supports | for inline concat or concat())
    let data = parse_and_format_args(args, input_signals, intermediate_signals)?;

    // Compute hash
    hash(algorithm, &data)
}

/// Execute hex encoding
fn execute_hex_encode(
    args: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let data = parse_and_format_args(args, input_signals, intermediate_signals)?;
    Ok(hex::encode(data).into_bytes())
}

/// Execute base64 encoding
fn execute_base64_encode(
    args: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    use base64::{Engine as _, engine::general_purpose};
    let data = parse_and_format_args(args, input_signals, intermediate_signals)?;
    Ok(general_purpose::STANDARD.encode(data).into_bytes())
}

/// Execute base58 encoding
fn execute_base58_encode(
    args: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let data = parse_and_format_args(args, input_signals, intermediate_signals)?;
    Ok(bs58::encode(data).into_vec())
}

/// Execute concatenation
fn execute_concat(
    args: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    // concat() uses comma-separated arguments
    let parts: Vec<&str> = args.split(',').collect();
    let mut output = Vec::new();

    for part in parts {
        let formatted = parse_and_format_args(part.trim(), input_signals, intermediate_signals)?;
        output.extend(formatted);
    }

    Ok(output)
}

/// Parse and format arguments with format specifiers
///
/// Supports:
/// - Single variable: `A{%x}`
/// - Inline concat with |: `A{%x}|B{%d}`
/// - Nested concat(): `concat(A{%x}, B{%d})`
fn parse_and_format_args(
    args: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();

    // Split by | for inline concatenation (only if not inside nested function)
    let parts: Vec<&str> = if args.contains("concat(") {
        // Has nested concat, don't split by |
        vec![args]
    } else {
        // Split by | for inline concat
        args.split('|').collect()
    };

    for part in parts {
        let part = part.trim();

        // Check if this is a nested function call
        if part.starts_with("concat(") && part.ends_with(')') {
            let inner_args = &part[7..part.len()-1];
            let nested_output = execute_concat(inner_args, input_signals, intermediate_signals)?;
            output.extend(nested_output);
        } else {
            // Parse variable and format specifier: A{%x} or just A
            let formatted = format_variable(part, input_signals, intermediate_signals)?;
            output.extend(formatted);
        }
    }

    Ok(output)
}

/// Format a single variable with optional format specifier
///
/// # Examples
///
/// - `A` - raw bytes
/// - `A{%x}` - hex lowercase
/// - `A{%08x}` - zero-padded hex
/// - `A{%064b64}` - zero-padded base64
fn format_variable(
    input: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    // Parse: variable_name{format_spec} or just variable_name
    if let Some(start) = input.find('{') {
        if !input.ends_with('}') {
            return Err(format!("Invalid format specifier: {}", input));
        }

        let var_name = input[..start].trim();
        let format_spec = &input[start+1..input.len()-1];

        // Get signal value
        let value = get_signal_value(var_name, input_signals, intermediate_signals)?;

        // Format according to specifier
        format_value(&value, format_spec)
    } else {
        // No format specifier, return raw bytes
        let var_name = input.trim();
        get_signal_value(var_name, input_signals, intermediate_signals)
    }
}

/// Get signal value by name from input or intermediate signals
fn get_signal_value(
    name: &str,
    input_signals: &HashMap<String, Vec<u8>>,
    intermediate_signals: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    // Check intermediate signals first (they override inputs)
    if let Some(value) = intermediate_signals.get(name) {
        return Ok(value.clone());
    }

    // Check input signals
    if let Some(value) = input_signals.get(name) {
        return Ok(value.clone());
    }

    Err(format!("Signal '{}' not found", name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_statement_sha256() {
        let mut signals = HashMap::new();
        signals.insert("A".to_string(), vec![255]);

        let (name, output) = execute_statement(
            "hash<==sha256(A{%x})",
            &signals,
            &HashMap::new()
        ).unwrap();

        assert_eq!(name, "hash");
        assert_eq!(output.len(), 32); // SHA-256 outputs 32 bytes
    }

    #[test]
    fn test_execute_concat() {
        let mut signals = HashMap::new();
        signals.insert("A".to_string(), vec![10]);
        signals.insert("B".to_string(), vec![20]);

        let (name, output) = execute_statement(
            "combined<==concat(A{%x}, B{%x})",
            &signals,
            &HashMap::new()
        ).unwrap();

        assert_eq!(name, "combined");
        // Should be "0a14" as bytes
        assert_eq!(String::from_utf8(output).unwrap(), "0a14");
    }
}