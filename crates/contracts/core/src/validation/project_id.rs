//! Project ID Validation Module
//!
//! Provides validation for project identifiers used to map donations to specific projects.
//! Supports alphanumeric IDs with hyphens and underscores.

use soroban_sdk::{Env, String};

/// Maximum length for a project ID
pub const MAX_PROJECT_ID_LENGTH: u32 = 64;

/// Minimum length for a project ID
pub const MIN_PROJECT_ID_LENGTH: u32 = 3;

/// Validation result for project IDs
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectIdValidationResult {
    Valid,
    Empty,
    TooShort,
    TooLong,
    InvalidCharacters,
    InvalidFormat,
}

impl ProjectIdValidationResult {
    /// Check if the validation result is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, ProjectIdValidationResult::Valid)
    }

    /// Get error message for the validation result
    pub fn message(&self) -> &'static str {
        match self {
            ProjectIdValidationResult::Valid => "Valid project ID",
            ProjectIdValidationResult::Empty => "Project ID cannot be empty",
            ProjectIdValidationResult::TooShort => "Project ID is too short (minimum 3 characters)",
            ProjectIdValidationResult::TooLong => "Project ID is too long (maximum 64 characters)",
            ProjectIdValidationResult::InvalidCharacters => "Project ID contains invalid characters (allowed: alphanumeric, hyphens, underscores)",
            ProjectIdValidationResult::InvalidFormat => "Project ID has invalid format (must start with alphanumeric)",
        }
    }
}

/// Validate a project ID format
///
/// Rules:
/// - Must not be empty
/// - Length: 3-64 characters
/// - Can contain: alphanumeric (A-Z, a-z, 0-9), hyphens (-), underscores (_)
/// - Must start with alphanumeric character
/// - Case-sensitive
pub fn validate_project_id(env: &Env, project_id: &String) -> ProjectIdValidationResult {
    // Check if empty
    if project_id.is_empty() {
        return ProjectIdValidationResult::Empty;
    }

    let len = project_id.len();

    // Check minimum length
    if len < MIN_PROJECT_ID_LENGTH {
        return ProjectIdValidationResult::TooShort;
    }

    // Check maximum length
    if len > MAX_PROJECT_ID_LENGTH {
        return ProjectIdValidationResult::TooLong;
    }

    // Check first character is alphanumeric
    let first_char = project_id.get(0);
    if !is_alphanumeric(first_char) {
        return ProjectIdValidationResult::InvalidFormat;
    }

    // Check all characters are valid
    for i in 0..len {
        let ch = project_id.get(i);
        if !is_valid_project_id_char(ch) {
            return ProjectIdValidationResult::InvalidCharacters;
        }
    }

    ProjectIdValidationResult::Valid
}

/// Check if a character is alphanumeric
fn is_alphanumeric(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9')
}

/// Check if a character is valid for project ID
/// Valid: alphanumeric, hyphen, underscore
fn is_valid_project_id_char(ch: char) -> bool {
    is_alphanumeric(ch) || ch == '-' || ch == '_'
}

/// Convenience function that returns boolean
pub fn is_valid_project_id(env: &Env, project_id: &String) -> bool {
    validate_project_id(env, project_id).is_valid()
}

/// Sanitize project ID by trimming and normalizing
/// Returns None if the project ID is invalid after sanitization
pub fn sanitize_project_id(env: &Env, project_id: &String) -> Option<String> {
    // For now, just validate as-is
    // In future, could implement trimming of whitespace, etc.
    if is_valid_project_id(env, project_id) {
        Some(project_id.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_valid_project_ids() {
        let env = Env::default();

        // Standard alphanumeric
        let id1 = String::from_str(&env, "PROJ123");
        assert_eq!(validate_project_id(&env, &id1), ProjectIdValidationResult::Valid);

        // With hyphens
        let id2 = String::from_str(&env, "proj-123");
        assert_eq!(validate_project_id(&env, &id2), ProjectIdValidationResult::Valid);

        // With underscores
        let id3 = String::from_str(&env, "proj_123");
        assert_eq!(validate_project_id(&env, &id3), ProjectIdValidationResult::Valid);

        // Mixed case
        let id4 = String::from_str(&env, "Proj-123_test");
        assert_eq!(validate_project_id(&env, &id4), ProjectIdValidationResult::Valid);

        // Minimum length (3 chars)
        let id5 = String::from_str(&env, "ABC");
        assert_eq!(validate_project_id(&env, &id5), ProjectIdValidationResult::Valid);
    }

    #[test]
    fn test_empty_project_id() {
        let env = Env::default();
        let id = String::from_str(&env, "");
        assert_eq!(validate_project_id(&env, &id), ProjectIdValidationResult::Empty);
    }

    #[test]
    fn test_too_short_project_id() {
        let env = Env::default();
        let id = String::from_str(&env, "AB");
        assert_eq!(validate_project_id(&env, &id), ProjectIdValidationResult::TooShort);
    }

    #[test]
    fn test_too_long_project_id() {
        let env = Env::default();
        // Create a 65-character string
        let long_id = "A".repeat(65);
        let id = String::from_str(&env, &long_id);
        assert_eq!(validate_project_id(&env, &id), ProjectIdValidationResult::TooLong);
    }

    #[test]
    fn test_invalid_starting_character() {
        let env = Env::default();
        // Starts with hyphen
        let id1 = String::from_str(&env, "-proj123");
        assert_eq!(validate_project_id(&env, &id1), ProjectIdValidationResult::InvalidFormat);

        // Starts with underscore
        let id2 = String::from_str(&env, "_proj123");
        assert_eq!(validate_project_id(&env, &id2), ProjectIdValidationResult::InvalidFormat);
    }

    #[test]
    fn test_invalid_characters() {
        let env = Env::default();
        // Contains space
        let id1 = String::from_str(&env, "proj 123");
        assert_eq!(validate_project_id(&env, &id1), ProjectIdValidationResult::InvalidCharacters);

        // Contains special character
        let id2 = String::from_str(&env, "proj@123");
        assert_eq!(validate_project_id(&env, &id2), ProjectIdValidationResult::InvalidCharacters);

        // Contains dot
        let id3 = String::from_str(&env, "proj.123");
        assert_eq!(validate_project_id(&env, &id3), ProjectIdValidationResult::InvalidCharacters);
    }

    #[test]
    fn test_is_valid_project_id_convenience() {
        let env = Env::default();
        
        let valid = String::from_str(&env, "valid-project-123");
        assert!(is_valid_project_id(&env, &valid));

        let invalid = String::from_str(&env, "");
        assert!(!is_valid_project_id(&env, &invalid));
    }
}
