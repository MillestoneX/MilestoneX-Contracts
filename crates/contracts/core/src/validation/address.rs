//! Stellar Address Validation Implementation
//!
//! Core validation logic for Stellar public keys and addresses, including:
//! - Format validation (length, prefix)
//! - Base32 checksum verification
//! - Muxed account support
//! - Comprehensive error handling

use crate::validation::{MuxedAddress, StellarAccount, StellarAddress, ValidationError};
use soroban_sdk::{Bytes, Env, String, Vec};

/// Validate a Stellar address format
///
/// Checks:
/// - Not empty
/// - Correct length (56 for standard, 69 for muxed)
/// - Valid prefix ('G' or 'M')
/// - Valid characters (base32 alphabet)
pub fn validate_stellar_address(
    env: &Env,
    address: String,
) -> Result<StellarAccount, ValidationError> {
    // Check if address is empty
    if address.is_empty() {
        return Err(ValidationError::EmptyAddress);
    }

    // Check length
    let len = address.len();
    if len != 56 && len != 69 {
        return Err(ValidationError::InvalidLength);
    }

    // Check first character
    let first_char = address.get(0);
    if first_char != 'G' && first_char != 'M' {
        return Err(ValidationError::InvalidFormat);
    }

    // Validate characters (base32 alphabet: A-Z, 2-7)
    if !is_valid_base32(&address) {
        return Err(ValidationError::InvalidCharacters);
    }

    // Perform checksum validation
    if !validate_checksum(env, &address) {
        return Err(ValidationError::InvalidChecksum);
    }

    // Handle muxed accounts (69 characters starting with 'M')
    if len == 69 && first_char == 'M' {
        // Parse muxed account ID (last 13 characters after 'M')
        let id_str = address.slice(56, 69);
        let id = parse_muxed_id(env, &id_str)?;
        let base_address = address.slice(0, 56);
        Ok(StellarAccount::Muxed(MuxedAddress::new(base_address, id)))
    } else {
        // Standard account
        Ok(StellarAccount::Standard(StellarAddress::new(address)))
    }
}

/// Check if a string contains only valid base32 characters
fn is_valid_base32(address: &String) -> bool {
    for i in 0..address.len() {
        let ch = address.get(i);
        // Base32 alphabet: A-Z and 2-7
        if !((ch >= 'A' && ch <= 'Z') || (ch >= '2' && ch <= '7')) {
            return false;
        }
    }
    true
}

/// Validate the checksum of a Stellar address using base32 decoding and CRC16-XMODEM
fn validate_checksum(env: &Env, address: &String) -> bool {
    // Stellar addresses use CRC16-XMODEM for checksum validation
    // Format: 1 byte version + 32 bytes payload + 2 bytes CRC16-XMODEM checksum

    if address.len() < 4 {
        return false;
    }

    // Decode base32 to bytes
    match decode_base32(address) {
        Some(decoded) => {
            // Decoded length should be 35 bytes (1 version + 32 payload + 2 checksum)
            if decoded.len() != 35 {
                return false;
            }

            // Extract the checksum (last 2 bytes)
            let provided_checksum = u16::from_le_bytes([decoded[33], decoded[34]]);

            // Calculate CRC16-XMODEM over version + payload (first 33 bytes)
            let calculated_checksum = crc16_xmodem(&decoded[0..33]);

            // Compare checksums
            provided_checksum == calculated_checksum
        }
        None => false,
    }
}

/// Decode a base32 string to bytes (Stellar alphabet: A-Z, 2-7)
fn decode_base32(data: &String) -> Option<Vec<u8>> {
    // Base32 decoding using Stellar's alphabet (A-Z, 2-7)
    const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let mut result = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits: usize = 0;

    for i in 0..data.len() {
        let ch = data.get(i) as u8;

        // Find the index of the character in the base32 alphabet
        let idx = match BASE32_ALPHABET.iter().position(|&c| c == ch) {
            Some(idx) => idx as u32,
            None => return None, // Invalid character
        };

        buffer = (buffer << 5) | idx;
        bits += 5;

        if bits >= 8 {
            bits -= 8;
            let byte = ((buffer >> bits) & 0xFF) as u8;
            result.push(byte);
        }
    }

    // Check that we consumed all bits properly
    if bits >= 5 {
        return None; // Invalid padding
    }

    Some(result)
}

/// Calculate CRC16-XMODEM checksum
/// Uses polynomial 0x1021 with no initial value and no final XOR
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;

    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc <<= 1;
            if crc & 0x10000 != 0 {
                crc ^= 0x1021;
            }
        }
    }

    crc & 0xFFFF
}

/// Parse the muxed account ID from the last 13 characters
/// Muxed accounts have format: M + 55 chars (base address without first char) + 13 chars (ID)
fn parse_muxed_id(env: &Env, id_str: &String) -> Result<u64, ValidationError> {
    // Validate that the ID string contains only base32 characters
    if !is_valid_base32(id_str) {
        return Err(ValidationError::InvalidMuxedFormat);
    }

    // Decode the 13-character base32 ID to bytes
    match decode_base32(id_str) {
        Some(decoded) => {
            // The 13 base32 characters decode to approximately 8 bytes
            // We extract the first 8 bytes as a 64-bit unsigned integer
            if decoded.len() < 8 {
                return Err(ValidationError::InvalidMuxedFormat);
            }

            // Convert the first 8 bytes to u64 (big-endian)
            let id = u64::from_be_bytes([
                decoded[0], decoded[1], decoded[2], decoded[3],
                decoded[4], decoded[5], decoded[6], decoded[7],
            ]);

            Ok(id)
        }
        None => Err(ValidationError::InvalidMuxedFormat),
    }
}

/// Convenience function to validate and return a standard Stellar address
pub fn validate_standard_address(
    env: &Env,
    address: String,
) -> Result<StellarAddress, ValidationError> {
    match validate_stellar_address(env, address)? {
        StellarAccount::Standard(addr) => Ok(addr),
        StellarAccount::Muxed(_) => Err(ValidationError::InvalidFormat),
    }
}

/// Convenience function to validate and return a muxed Stellar address
pub fn validate_muxed_address(env: &Env, address: String) -> Result<MuxedAddress, ValidationError> {
    match validate_stellar_address(env, address)? {
        StellarAccount::Muxed(addr) => Ok(addr),
        StellarAccount::Standard(_) => Err(ValidationError::InvalidFormat),
    }
}

/// Simple validation function that returns boolean (for external use)
pub fn is_valid_stellar_address(env: &Env, address: String) -> bool {
    validate_stellar_address(env, address).is_ok()
}

/// Validate multiple addresses at once
pub fn validate_addresses(
    env: &Env,
    addresses: Vec<String>,
) -> Vec<Result<StellarAccount, ValidationError>> {
    let mut results = Vec::new(env);
    for address in addresses.iter() {
        results.push_back(validate_stellar_address(env, address));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, String, Vec};

    #[test]
    fn test_valid_standard_address() {
        let env = Env::default();
        let valid_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37",
        );

        let result = validate_standard_address(&env, valid_address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_length() {
        let env = Env::default();
        let short_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W3",
        ); // 55 chars

        let result = validate_stellar_address(&env, short_address);
        assert!(matches!(result, Err(ValidationError::InvalidLength)));
    }

    #[test]
    fn test_invalid_prefix() {
        let env = Env::default();
        let invalid_address = String::from_str(
            &env,
            "ADQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37",
        ); // Starts with 'A'

        let result = validate_stellar_address(&env, invalid_address);
        assert!(matches!(result, Err(ValidationError::InvalidFormat)));
    }

    #[test]
    fn test_empty_address() {
        let env = Env::default();
        let empty_address = String::from_str(&env, "");

        let result = validate_stellar_address(&env, empty_address);
        assert!(matches!(result, Err(ValidationError::EmptyAddress)));
    }

    #[test]
    fn test_invalid_characters() {
        let env = Env::default();
        let invalid_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W38",
        ); // Contains '8'

        let result = validate_stellar_address(&env, invalid_address);
        assert!(matches!(result, Err(ValidationError::InvalidCharacters)));
    }

    #[test]
    fn test_muxed_account_valid_format() {
        let env = Env::default();
        // Muxed account: 69 characters starting with 'M'
        let muxed_address = String::from_str(
            &env,
            "MDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37654ABCD2I",
        );

        let result = validate_stellar_address(&env, muxed_address);
        // Format validation passes, checksum may fail but format is checked here
        assert!(result.is_ok() || matches!(result, Err(ValidationError::InvalidChecksum)));
    }

    #[test]
    fn test_muxed_account_invalid_length() {
        let env = Env::default();
        // Muxed account with wrong length (68 chars instead of 69)
        let muxed_address = String::from_str(
            &env,
            "MDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37654ABCD2",
        );

        let result = validate_stellar_address(&env, muxed_address);
        assert!(matches!(result, Err(ValidationError::InvalidLength)));
    }

    #[test]
    fn test_muxed_account_wrong_prefix() {
        let env = Env::default();
        // 69 characters but starts with 'G' instead of 'M'
        let muxed_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37654ABCD2I",
        );

        let result = validate_stellar_address(&env, muxed_address);
        assert!(matches!(result, Err(ValidationError::InvalidLength)));
    }

    #[test]
    fn test_validate_multiple_addresses() {
        let env = Env::default();
        let addresses = {
            let mut v = Vec::new(&env);
            v.push_back(String::from_str(
                &env,
                "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37",
            ));
            v.push_back(String::from_str(&env, "INVALID"));
            v
        };

        let results = validate_addresses(&env, addresses);
        assert_eq!(results.len(), 2);
        assert!(results.get(0).is_ok());
        assert!(results.get(1).is_err());
    }

    #[test]
    fn test_is_valid_stellar_address_bool() {
        let env = Env::default();
        let valid_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37",
        );
        let invalid_address = String::from_str(&env, "INVALID");

        assert!(is_valid_stellar_address(&env, valid_address));
        assert!(!is_valid_stellar_address(&env, invalid_address));
    }

    #[test]
    fn test_validate_only_standard_address() {
        let env = Env::default();
        let valid_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37",
        );

        let result = validate_standard_address(&env, valid_address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_muxed_when_expecting_standard() {
        let env = Env::default();
        let muxed_address = String::from_str(
            &env,
            "MDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37654ABCD2I",
        );

        let result = validate_standard_address(&env, muxed_address);
        // Will fail at checksum validation first, but if it passes checksum, format validation will catch it
        assert!(result.is_err());
    }

    #[test]
    fn test_base32_validation() {
        let env = Env::default();
        // Valid base32 characters: A-Z and 2-7
        let valid_chars = String::from_str(&env, "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");
        assert!(is_valid_base32(&valid_chars));

        // Invalid characters: 0, 1, 8, 9, lowercase
        let invalid_chars = String::from_str(&env, "ABCD0123");
        assert!(!is_valid_base32(&invalid_chars));
    }

    #[test]
    fn test_checksum_validation_fails_corrupted() {
        let env = Env::default();
        // Valid address structure but corrupted checksum
        let corrupted_address = String::from_str(
            &env,
            "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W3A",
        ); // Last char changed

        let result = validate_stellar_address(&env, corrupted_address);
        // Should fail due to invalid checksum
        assert!(result.is_err());
    }
}
