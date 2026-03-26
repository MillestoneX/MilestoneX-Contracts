//! Stellar Address Validation Errors
//!
//! Comprehensive error types for all validation failures with descriptive messages.

use soroban_sdk::{contracterror, panic_with_error};

/// Stellar address validation errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ValidationError {
    /// Address is empty or null
    EmptyAddress = 1,

    /// Invalid address length (expected 56 for standard, 69 for muxed)
    InvalidLength = 2,

    /// Address format is invalid (must start with 'G' or 'M')
    InvalidFormat = 3,

    /// Checksum verification failed
    InvalidChecksum = 4,

    /// Invalid base32 encoding
    InvalidEncoding = 5,

    /// Muxed account parsing failed
    InvalidMuxedFormat = 6,

    /// Address contains invalid characters
    InvalidCharacters = 7,

    /// Unsupported address version
    UnsupportedVersion = 8,

    /// Project ID is empty or missing
    EmptyProjectId = 9,

    /// Project ID is too short (minimum 3 characters)
    ProjectIdTooShort = 10,

    /// Project ID is too long (maximum 64 characters)
    ProjectIdTooLong = 11,

    /// Project ID contains invalid characters
    InvalidProjectIdCharacters = 12,

    /// Project ID has invalid format
    InvalidProjectIdFormat = 13,

    /// Duplicate transaction hash detected
    DuplicateTransaction = 14,
}

impl ValidationError {
    /// Get a descriptive error message
    pub fn message(&self) -> &'static str {
        match self {
            ValidationError::EmptyAddress => "Address cannot be empty",
            ValidationError::InvalidLength => "Invalid address length - must be 56 characters for standard accounts or 69 for muxed accounts",
            ValidationError::InvalidFormat => "Invalid address format - must start with 'G' for standard accounts or 'M' for muxed accounts",
            ValidationError::InvalidChecksum => "Address checksum verification failed",
            ValidationError::InvalidEncoding => "Invalid base32 encoding in address",
            ValidationError::InvalidMuxedFormat => "Invalid muxed account format",
            ValidationError::InvalidCharacters => "Address contains invalid characters",
            ValidationError::UnsupportedVersion => "Unsupported Stellar address version",
            ValidationError::EmptyProjectId => "Project ID cannot be empty",
            ValidationError::ProjectIdTooShort => "Project ID is too short (minimum 3 characters)",
            ValidationError::ProjectIdTooLong => "Project ID is too long (maximum 64 characters)",
            ValidationError::InvalidProjectIdCharacters => "Project ID contains invalid characters (allowed: alphanumeric, hyphens, underscores)",
            ValidationError::InvalidProjectIdFormat => "Project ID has invalid format (must start with alphanumeric)",
            ValidationError::DuplicateTransaction => "Duplicate transaction hash detected - this transaction has already been processed",
        }
    }

    /// Panic with this error
    pub fn panic<E: soroban_sdk::Env>(self, env: &E) -> ! {
        panic_with_error!(env, self)
    }
}
