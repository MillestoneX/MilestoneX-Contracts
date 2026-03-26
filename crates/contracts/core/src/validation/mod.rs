//! Stellar Address Validation Utilities
//!
//! This module provides comprehensive validation for Stellar public keys and addresses,
//! including format validation, checksum verification, and support for both standard
//! and muxed accounts. Also includes project ID validation for donation mapping.

pub mod address;
pub mod errors;
pub mod project_id;
pub mod types;

pub use address::*;
pub use errors::*;
pub use project_id::*;
pub use types::*;
