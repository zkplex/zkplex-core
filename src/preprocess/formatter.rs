//! Value formatting with printf-style format specifiers
//!
//! Supports:
//! - `%x` / `%X` - hex lowercase/uppercase
//! - `%d` - decimal
//! - `%o` - octal
//! - `%08x` - zero-padded hex (8 chars)
//! - `%b64` / `%B64` - base64 lowercase/uppercase
//! - `%b58` / `%B58` - base58 lowercase/uppercase
//! - `%064b64` - zero-padded base64 (64 chars)
//! - `%032b58` - zero-padded base58 (32 chars)

use base64::{Engine as _, engine::general_purpose};

/// Format a value according to a format specifier
///
/// # Arguments
///
/// * `value` - Raw bytes to format
/// * `spec` - Format specifier (e.g., "%x", "%08x", "%064b64")
///
/// # Returns
///
/// Formatted value as bytes (UTF-8 string)
pub fn format_value(value: &[u8], spec: &str) -> Result<Vec<u8>, String> {
    if !spec.starts_with('%') {
        return Err(format!("Format specifier must start with %: {}", spec));
    }

    let spec = &spec[1..]; // Remove leading %

    // Parse padding and format type
    let (padding, format_type) = parse_format_spec(spec)?;

    // Format the value
    let formatted = match format_type {
        FormatType::Hex { uppercase } => format_hex(value, uppercase),
        FormatType::Decimal => format_decimal(value),
        FormatType::Octal => format_octal(value),
        FormatType::Base64 { uppercase } => format_base64(value, uppercase),
        FormatType::Base58 { uppercase } => format_base58(value, uppercase),
    };

    // Apply padding if specified
    let output = if let Some(width) = padding {
        apply_padding(&formatted, width)
    } else {
        formatted
    };

    Ok(output.into_bytes())
}

/// Format type
#[derive(Debug, PartialEq)]
enum FormatType {
    Hex { uppercase: bool },
    Decimal,
    Octal,
    Base64 { uppercase: bool },
    Base58 { uppercase: bool },
}

/// Parse format specification into padding and format type
///
/// # Examples
///
/// - "x" -> (None, Hex{false})
/// - "08x" -> (Some(8), Hex{false})
/// - "X" -> (None, Hex{true})
/// - "064b64" -> (Some(64), Base64{false})
fn parse_format_spec(spec: &str) -> Result<(Option<usize>, FormatType), String> {
    if spec.is_empty() {
        return Err("Empty format specifier".to_string());
    }

    // Check for base64/base58 first (they can have digits in the name)
    if spec.ends_with("b64") {
        let width_str = &spec[..spec.len()-3];
        return Ok((parse_padding(width_str)?, FormatType::Base64 { uppercase: false }));
    }
    if spec.ends_with("B64") {
        let width_str = &spec[..spec.len()-3];
        return Ok((parse_padding(width_str)?, FormatType::Base64 { uppercase: true }));
    }
    if spec.ends_with("b58") {
        let width_str = &spec[..spec.len()-3];
        return Ok((parse_padding(width_str)?, FormatType::Base58 { uppercase: false }));
    }
    if spec.ends_with("B58") {
        let width_str = &spec[..spec.len()-3];
        return Ok((parse_padding(width_str)?, FormatType::Base58 { uppercase: true }));
    }

    // Parse single-char formats (x, X, d, o)
    let last_char = spec.chars().last().unwrap();
    let width_str = &spec[..spec.len()-1];

    let format_type = match last_char {
        'x' => FormatType::Hex { uppercase: false },
        'X' => FormatType::Hex { uppercase: true },
        'd' => FormatType::Decimal,
        'o' => FormatType::Octal,
        _ => return Err(format!("Unknown format type: {}", last_char)),
    };

    Ok((parse_padding(width_str)?, format_type))
}

/// Parse padding width from format string
fn parse_padding(s: &str) -> Result<Option<usize>, String> {
    if s.is_empty() {
        return Ok(None);
    }

    // Handle leading zero for zero-padding
    let s = if s.starts_with('0') { &s[1..] } else { s };

    if s.is_empty() {
        return Ok(None);
    }

    s.parse::<usize>()
        .map(Some)
        .map_err(|_| format!("Invalid padding width: {}", s))
}

/// Format bytes as hexadecimal
fn format_hex(value: &[u8], uppercase: bool) -> String {
    if uppercase {
        hex::encode_upper(value)
    } else {
        hex::encode(value)
    }
}

/// Format bytes as decimal number (interprets as big-endian integer)
fn format_decimal(value: &[u8]) -> String {
    if value.is_empty() {
        return "0".to_string();
    }

    // Convert bytes to decimal string (big-endian)
    use num_bigint::BigUint;
    let num = BigUint::from_bytes_be(value);
    num.to_string()
}

/// Format bytes as octal number (interprets as big-endian integer)
fn format_octal(value: &[u8]) -> String {
    if value.is_empty() {
        return "0".to_string();
    }

    use num_bigint::BigUint;
    let num = BigUint::from_bytes_be(value);
    num.to_str_radix(8)
}

/// Format bytes as base64
fn format_base64(value: &[u8], uppercase: bool) -> String {
    let encoded = general_purpose::STANDARD.encode(value);
    if uppercase {
        encoded.to_uppercase()
    } else {
        encoded
    }
}

/// Format bytes as base58
fn format_base58(value: &[u8], uppercase: bool) -> String {
    let encoded = bs58::encode(value).into_string();
    if uppercase {
        encoded.to_uppercase()
    } else {
        encoded
    }
}

/// Apply zero-padding to a string
fn apply_padding(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let padding = width - s.len();
        format!("{}{}", "0".repeat(padding), s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex_lowercase() {
        let value = vec![255, 16, 32];
        let result = format_value(&value, "%x").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "ff1020");
    }

    #[test]
    fn test_format_hex_uppercase() {
        let value = vec![255, 16, 32];
        let result = format_value(&value, "%X").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "FF1020");
    }

    #[test]
    fn test_format_hex_padded() {
        let value = vec![255];
        let result = format_value(&value, "%08x").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "000000ff");
    }

    #[test]
    fn test_format_decimal() {
        let value = vec![0, 255]; // 255 in big-endian
        let result = format_value(&value, "%d").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "255");
    }

    #[test]
    fn test_format_octal() {
        let value = vec![8]; // 8 decimal = 10 octal
        let result = format_value(&value, "%o").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "10");
    }

    #[test]
    fn test_format_base64() {
        let value = b"hello";
        let result = format_value(value, "%b64").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "aGVsbG8=");
    }

    //noinspection ALL
    #[test]
    fn test_format_base64_uppercase() {
        let value = b"hello";
        let result = format_value(value, "%B64").unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "AGVSBG8=");
    }

    #[test]
    fn test_format_base64_padded() {
        let value = b"hi";
        let result = format_value(value, "%010b64").unwrap();
        let s = String::from_utf8(result).unwrap();
        assert_eq!(s.len(), 10);
        assert!(s.starts_with("000000"));
    }

    #[test]
    fn test_parse_format_spec() {
        assert_eq!(parse_format_spec("x").unwrap(), (None, FormatType::Hex { uppercase: false }));
        assert_eq!(parse_format_spec("08x").unwrap(), (Some(8), FormatType::Hex { uppercase: false }));
        assert_eq!(parse_format_spec("X").unwrap(), (None, FormatType::Hex { uppercase: true }));
        assert_eq!(parse_format_spec("064b64").unwrap(), (Some(64), FormatType::Base64 { uppercase: false }));
        assert_eq!(parse_format_spec("d").unwrap(), (None, FormatType::Decimal));
    }
}