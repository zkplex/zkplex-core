//! Hash function implementations
//!
//! Supports multiple cryptographic hash algorithms:
//! - SHA-1, SHA-256, SHA-512
//! - SHA3-256, SHA3-512 (Standard SHA3)
//! - MD5
//! - CRC32
//! - BLAKE2b, BLAKE3
//! - Keccak-256 (Ethereum)
//! - RIPEMD-160 (Bitcoin)

use digest::Digest;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use md5::Md5;
use blake2::{Blake2b, digest::consts::U32};
use sha3::{Keccak256, Sha3_256, Sha3_512};
use blake3::Hasher as Blake3Hasher;
use ripemd::Ripemd160;

/// Supported hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    SHA1,
    SHA256,
    SHA512,
    SHA3_256,
    SHA3_512,
    MD5,
    CRC32,
    BLAKE2b,
    BLAKE3,
    Keccak256,
    RIPEMD160,
}

/// Compute hash of data using specified algorithm
///
/// # Arguments
///
/// * `algorithm` - Hash algorithm to use
/// * `data` - Input data to hash
///
/// # Returns
///
/// Hash output as bytes
///
/// # Example
///
/// ```ignore
/// let data = b"hello world";
/// let hash = hash(HashAlgorithm::SHA256, data)?;
/// assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
/// ```
pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> Result<Vec<u8>, String> {
    match algorithm {
        HashAlgorithm::SHA1 => Ok(hash_sha1(data)),
        HashAlgorithm::SHA256 => Ok(hash_sha256(data)),
        HashAlgorithm::SHA512 => Ok(hash_sha512(data)),
        HashAlgorithm::SHA3_256 => Ok(hash_sha3_256(data)),
        HashAlgorithm::SHA3_512 => Ok(hash_sha3_512(data)),
        HashAlgorithm::MD5 => Ok(hash_md5(data)),
        HashAlgorithm::CRC32 => Ok(hash_crc32(data)),
        HashAlgorithm::BLAKE2b => Ok(hash_blake2b(data)),
        HashAlgorithm::BLAKE3 => Ok(hash_blake3(data)),
        HashAlgorithm::Keccak256 => Ok(hash_keccak256(data)),
        HashAlgorithm::RIPEMD160 => Ok(hash_ripemd160(data)),
    }
}

/// Compute SHA-1 hash (20 bytes)
fn hash_sha1(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute SHA-256 hash (32 bytes)
fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute SHA-512 hash (64 bytes)
fn hash_sha512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute MD5 hash (16 bytes)
fn hash_md5(data: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute CRC32 checksum (4 bytes)
fn hash_crc32(data: &[u8]) -> Vec<u8> {
    let checksum = crc32fast::hash(data);
    checksum.to_be_bytes().to_vec()
}

/// Compute BLAKE2b hash (32 bytes, truncated from 64)
fn hash_blake2b(data: &[u8]) -> Vec<u8> {
    type Blake2b256 = Blake2b<U32>;

    let mut hasher = Blake2b256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute SHA3-256 hash (32 bytes) - Standard SHA3
fn hash_sha3_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute SHA3-512 hash (64 bytes) - Standard SHA3
fn hash_sha3_512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute BLAKE3 hash (32 bytes)
fn hash_blake3(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake3Hasher::new();
    hasher.update(data);
    hasher.finalize().as_bytes().to_vec()
}

/// Compute Keccak-256 hash (32 bytes) - Ethereum style
fn hash_keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute RIPEMD-160 hash (20 bytes) - Bitcoin style
fn hash_ripemd160(data: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"hello";
        let result = hash(HashAlgorithm::SHA256, data).unwrap();
        assert_eq!(result.len(), 32);

        // Known SHA-256 hash of "hello"
        let expected = hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sha1() {
        let data = b"hello";
        let result = hash(HashAlgorithm::SHA1, data).unwrap();
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_sha512() {
        let data = b"hello";
        let result = hash(HashAlgorithm::SHA512, data).unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_md5() {
        let data = b"hello";
        let result = hash(HashAlgorithm::MD5, data).unwrap();
        assert_eq!(result.len(), 16);

        // Known MD5 hash of "hello"
        let expected = hex::decode("5d41402abc4b2a76b9719d911017c592").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_crc32() {
        let data = b"hello";
        let result = hash(HashAlgorithm::CRC32, data).unwrap();
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_blake2b() {
        let data = b"hello";
        let result = hash(HashAlgorithm::BLAKE2b, data).unwrap();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_keccak256() {
        let data = b"hello";
        let result = hash(HashAlgorithm::Keccak256, data).unwrap();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_sha3_256() {
        let data = b"hello";
        let result = hash(HashAlgorithm::SHA3_256, data).unwrap();
        assert_eq!(result.len(), 32);

        // Known SHA3-256 hash of "hello"
        let expected = hex::decode("3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392").unwrap();
        assert_eq!(result, expected);
    }

    //noinspection ALL
    #[test]
    fn test_sha3_512() {
        let data = b"hello";
        let result = hash(HashAlgorithm::SHA3_512, data).unwrap();
        assert_eq!(result.len(), 64);

        // Known SHA3-512 hash of "hello"
        let expected = hex::decode("75d527c368f2efe848ecf6b073a36767800805e9eef2b1857d5f984f036eb6df891d75f72d9b154518c1cd58835286d1da9a38deba3de98b5a53e5ed78a84976").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_blake3() {
        let data = b"hello";
        let result = hash(HashAlgorithm::BLAKE3, data).unwrap();
        assert_eq!(result.len(), 32);

        // Known BLAKE3 hash of "hello"
        let expected = hex::decode("ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_ripemd160() {
        let data = b"hello";
        let result = hash(HashAlgorithm::RIPEMD160, data).unwrap();
        assert_eq!(result.len(), 20);

        // Known RIPEMD-160 hash of "hello"
        let expected = hex::decode("108f07b8382412612c048d07d13f814118445acd").unwrap();
        assert_eq!(result, expected);
    }

    //noinspection ALL
    #[test]
    fn test_empty_input() {
        let data = b"";
        let result = hash(HashAlgorithm::SHA256, data).unwrap();
        assert_eq!(result.len(), 32);

        // SHA-256 of empty string
        let expected = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_all_algorithms() {
        // Test that all algorithms produce output of expected length
        let data = b"test data";

        assert_eq!(hash(HashAlgorithm::SHA1, data).unwrap().len(), 20);
        assert_eq!(hash(HashAlgorithm::SHA256, data).unwrap().len(), 32);
        assert_eq!(hash(HashAlgorithm::SHA512, data).unwrap().len(), 64);
        assert_eq!(hash(HashAlgorithm::SHA3_256, data).unwrap().len(), 32);
        assert_eq!(hash(HashAlgorithm::SHA3_512, data).unwrap().len(), 64);
        assert_eq!(hash(HashAlgorithm::MD5, data).unwrap().len(), 16);
        assert_eq!(hash(HashAlgorithm::CRC32, data).unwrap().len(), 4);
        assert_eq!(hash(HashAlgorithm::BLAKE2b, data).unwrap().len(), 32);
        assert_eq!(hash(HashAlgorithm::BLAKE3, data).unwrap().len(), 32);
        assert_eq!(hash(HashAlgorithm::Keccak256, data).unwrap().len(), 32);
        assert_eq!(hash(HashAlgorithm::RIPEMD160, data).unwrap().len(), 20);
    }
}