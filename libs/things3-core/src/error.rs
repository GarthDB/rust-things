//! Error types for the Things Core library

use thiserror::Error;

/// Result type alias for Things operations
pub type Result<T> = std::result::Result<T, ThingsError>;

/// Database-specific errors (introduced in 3.0).
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ThingsDatabaseError {
    #[deprecated(
        since = "2.1.0",
        note = "Migrate to a specific ThingsDatabaseError variant."
    )]
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database not found: {path}. Ensure Things 3 is installed and has been opened at least once, or specify a custom database path.")]
    DatabaseNotFound { path: String },

    #[error("Task not found: {uuid}. The task may have been deleted or moved. Try searching by title instead.")]
    TaskNotFound { uuid: String },

    #[error("Project not found: {uuid}. The project may have been deleted. Verify the UUID or list all projects to find the correct one.")]
    ProjectNotFound { uuid: String },

    #[error("Area not found: {uuid}. The area may have been deleted. Verify the UUID or list all areas to find the correct one.")]
    AreaNotFound { uuid: String },

    #[error("AppleScript automation failed: {message}")]
    AppleScript { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

/// Query-specific errors (introduced in 3.0).
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ThingsQueryError {
    #[error("Invalid UUID: {uuid}. UUIDs must be in format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")]
    InvalidUuid { uuid: String },

    #[error("Invalid date: {date}")]
    InvalidDate { date: String },

    #[error("Date validation failed: {0}")]
    DateValidation(#[from] crate::database::DateValidationError),

    #[error("Date conversion failed: {0}")]
    DateConversion(#[from] crate::database::DateConversionError),

    #[error("Invalid cursor: {0}")]
    InvalidCursor(String),
}

/// Export-specific errors (introduced in 3.0).
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ThingsExportError {
    #[error("Export IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Export serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Export format unavailable: {message}")]
    FormatUnavailable { message: String },
}

/// Main error type for Things operations
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ThingsError {
    #[error(transparent)]
    Database(#[from] ThingsDatabaseError),

    #[error(transparent)]
    Query(#[from] ThingsQueryError),

    #[error(transparent)]
    Export(#[from] ThingsExportError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[deprecated(
        since = "2.1.0",
        note = "Will be removed in 3.0. Use a specific variant such as ThingsError::Validation or ThingsError::configuration() instead."
    )]
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

// Two-step From conversions so `?` works at DB-layer call sites.
impl From<crate::database::DateValidationError> for ThingsError {
    fn from(e: crate::database::DateValidationError) -> Self {
        ThingsError::Query(ThingsQueryError::DateValidation(e))
    }
}

impl From<crate::database::DateConversionError> for ThingsError {
    fn from(e: crate::database::DateConversionError) -> Self {
        ThingsError::Query(ThingsQueryError::DateConversion(e))
    }
}

impl ThingsError {
    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Database(ThingsDatabaseError::Configuration {
            message: message.into(),
        })
    }

    /// Create an AppleScript error.
    pub fn applescript(message: impl Into<String>) -> Self {
        Self::Database(ThingsDatabaseError::AppleScript {
            message: message.into(),
        })
    }

    /// Create an unknown error.
    #[deprecated(
        since = "2.1.0",
        note = "Will be removed in 3.0. Use a specific constructor such as ThingsError::validation() or ThingsError::configuration() instead."
    )]
    #[allow(deprecated)]
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_serialization_error_from_serde() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let things_error: ThingsError = json_error.into();

        match things_error {
            ThingsError::Serialization(_) => (),
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_io_error_from_std() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let things_error: ThingsError = io_error.into();

        match things_error {
            ThingsError::Io(_) => (),
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_database_not_found_error() {
        let error: ThingsError = ThingsDatabaseError::DatabaseNotFound {
            path: "/path/to/db".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Database not found"));
        assert!(error.to_string().contains("/path/to/db"));
    }

    #[test]
    fn test_invalid_uuid_error() {
        let error: ThingsError = ThingsQueryError::InvalidUuid {
            uuid: "invalid-uuid".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Invalid UUID"));
        assert!(error.to_string().contains("invalid-uuid"));
    }

    #[test]
    fn test_invalid_date_error() {
        let error: ThingsError = ThingsQueryError::InvalidDate {
            date: "2023-13-45".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Invalid date"));
        assert!(error.to_string().contains("2023-13-45"));
    }

    #[test]
    fn test_task_not_found_error() {
        let error: ThingsError = ThingsDatabaseError::TaskNotFound {
            uuid: "task-uuid-123".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Task not found"));
        assert!(error.to_string().contains("task-uuid-123"));
    }

    #[test]
    fn test_project_not_found_error() {
        let error: ThingsError = ThingsDatabaseError::ProjectNotFound {
            uuid: "project-uuid-456".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Project not found"));
        assert!(error.to_string().contains("project-uuid-456"));
    }

    #[test]
    fn test_area_not_found_error() {
        let error: ThingsError = ThingsDatabaseError::AreaNotFound {
            uuid: "area-uuid-789".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Area not found"));
        assert!(error.to_string().contains("area-uuid-789"));
    }

    #[test]
    fn test_validation_error() {
        let error = ThingsError::Validation {
            message: "Invalid input data".to_string(),
        };

        assert!(error.to_string().contains("Validation error"));
        assert!(error.to_string().contains("Invalid input data"));
    }

    #[test]
    fn test_configuration_error() {
        let error: ThingsError = ThingsDatabaseError::Configuration {
            message: "Missing required config".to_string(),
        }
        .into();

        assert!(error.to_string().contains("Configuration error"));
        assert!(error.to_string().contains("Missing required config"));
    }

    #[test]
    fn test_unknown_error() {
        let error = ThingsError::Unknown {
            message: "Something went wrong".to_string(),
        };

        assert!(error.to_string().contains("Unknown error"));
        assert!(error.to_string().contains("Something went wrong"));
    }

    #[test]
    fn test_applescript_error() {
        let error: ThingsError = ThingsDatabaseError::AppleScript {
            message: "macOS Automation permission denied".to_string(),
        }
        .into();

        assert!(error.to_string().contains("AppleScript automation failed"));
        assert!(error
            .to_string()
            .contains("macOS Automation permission denied"));
    }

    #[test]
    fn test_applescript_helper() {
        let error = ThingsError::applescript("osascript not available");

        match error {
            ThingsError::Database(ThingsDatabaseError::AppleScript { message }) => {
                assert_eq!(message, "osascript not available");
            }
            _ => panic!("Expected AppleScript error"),
        }
    }

    #[test]
    fn test_validation_helper() {
        let error = ThingsError::validation("Test validation message");

        match error {
            ThingsError::Validation { message } => {
                assert_eq!(message, "Test validation message");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_helper_with_string() {
        let message = "Test validation message".to_string();
        let error = ThingsError::validation(message);

        match error {
            ThingsError::Validation { message } => {
                assert_eq!(message, "Test validation message");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_configuration_helper() {
        let error = ThingsError::configuration("Test config message");

        match error {
            ThingsError::Database(ThingsDatabaseError::Configuration { message }) => {
                assert_eq!(message, "Test config message");
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_configuration_helper_with_string() {
        let message = "Test config message".to_string();
        let error = ThingsError::configuration(message);

        match error {
            ThingsError::Database(ThingsDatabaseError::Configuration { message }) => {
                assert_eq!(message, "Test config message");
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_unknown_helper() {
        let error = ThingsError::unknown("Test unknown message");

        match error {
            ThingsError::Unknown { message } => {
                assert_eq!(message, "Test unknown message");
            }
            _ => panic!("Expected Unknown error"),
        }
    }

    #[test]
    fn test_unknown_helper_with_string() {
        let message = "Test unknown message".to_string();
        let error = ThingsError::unknown(message);

        match error {
            ThingsError::Unknown { message } => {
                assert_eq!(message, "Test unknown message");
            }
            _ => panic!("Expected Unknown error"),
        }
    }

    #[test]
    fn test_error_display_formatting() {
        let errors: Vec<ThingsError> = vec![
            ThingsDatabaseError::DatabaseNotFound {
                path: "test.db".to_string(),
            }
            .into(),
            ThingsQueryError::InvalidUuid {
                uuid: "bad-uuid".to_string(),
            }
            .into(),
            ThingsQueryError::InvalidDate {
                date: "bad-date".to_string(),
            }
            .into(),
            ThingsDatabaseError::TaskNotFound {
                uuid: "task-123".to_string(),
            }
            .into(),
            ThingsDatabaseError::ProjectNotFound {
                uuid: "project-456".to_string(),
            }
            .into(),
            ThingsDatabaseError::AreaNotFound {
                uuid: "area-789".to_string(),
            }
            .into(),
            ThingsError::Validation {
                message: "validation failed".to_string(),
            },
            ThingsDatabaseError::Configuration {
                message: "config error".to_string(),
            }
            .into(),
            ThingsError::Unknown {
                message: "unknown error".to_string(),
            },
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
            assert!(error_string.len() > 10);
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = ThingsError::Validation {
            message: "test message".to_string(),
        };

        let debug_string = format!("{error:?}");
        assert!(debug_string.contains("Validation"));
        assert!(debug_string.contains("test message"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_error() -> Result<String> {
            Err(ThingsError::validation("test error"))
        }

        assert!(returns_error().is_err());

        match returns_error() {
            Err(ThingsError::Validation { message }) => {
                assert_eq!(message, "test error");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_database_error_deprecated_variant() {
        let inner = ThingsDatabaseError::Database("raw db error".to_string());
        let error: ThingsError = inner.into();
        assert!(error.to_string().contains("Database error"));
    }

    #[test]
    fn test_from_impls() {
        let db_err = ThingsDatabaseError::DatabaseNotFound {
            path: "p".to_string(),
        };
        let _: ThingsError = db_err.into();

        let q_err = ThingsQueryError::InvalidUuid {
            uuid: "u".to_string(),
        };
        let _: ThingsError = q_err.into();

        let ex_err = ThingsExportError::FormatUnavailable {
            message: "csv not enabled".to_string(),
        };
        let _: ThingsError = ex_err.into();
    }

    #[test]
    fn test_invalid_cursor_error() {
        let error: ThingsError =
            ThingsQueryError::InvalidCursor("bad cursor data".to_string()).into();
        assert!(error.to_string().contains("Invalid cursor"));
        assert!(error.to_string().contains("bad cursor data"));
    }
}
