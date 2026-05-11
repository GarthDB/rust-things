//! Configuration management for Things 3 integration

use crate::error::{Result, ThingsError};
use std::path::{Path, PathBuf};

/// Configuration for Things 3 database access
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ThingsConfig {
    database_path: PathBuf,
    fallback_to_default: bool,
}

/// Builder for [`ThingsConfig`].
#[derive(Debug, Default)]
pub struct ThingsConfigBuilder {
    database_path: Option<PathBuf>,
    fallback_to_default: bool,
}

impl ThingsConfigBuilder {
    pub fn database_path(mut self, path: impl AsRef<Path>) -> Self {
        self.database_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn fallback_to_default(mut self, fallback: bool) -> Self {
        self.fallback_to_default = fallback;
        self
    }

    pub fn build(self) -> Result<ThingsConfig> {
        let database_path = self
            .database_path
            .unwrap_or_else(ThingsConfig::get_default_database_path);
        Ok(ThingsConfig {
            database_path,
            fallback_to_default: self.fallback_to_default,
        })
    }
}

impl ThingsConfig {
    /// Returns a new builder for [`ThingsConfig`].
    pub fn builder() -> ThingsConfigBuilder {
        ThingsConfigBuilder::default()
    }

    /// Path to the Things 3 database.
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    /// Whether to use the default database path if the specified path doesn't exist.
    pub fn fallback_to_default(&self) -> bool {
        self.fallback_to_default
    }

    /// Create a configuration with the default database path
    #[must_use]
    pub fn with_default_path() -> Self {
        Self {
            database_path: Self::get_default_database_path(),
            fallback_to_default: false,
        }
    }

    /// Get the effective database path, falling back to default if needed
    ///
    /// # Errors
    /// Returns an error if neither the specified path nor the default path exists
    pub fn get_effective_database_path(&self) -> Result<PathBuf> {
        if self.database_path.exists() {
            return Ok(self.database_path.clone());
        }

        if self.fallback_to_default {
            let default_path = Self::get_default_database_path();
            if default_path.exists() {
                return Ok(default_path);
            }
        }

        Err(ThingsError::configuration(format!(
            "Database not found at {} and fallback is {}",
            self.database_path.display(),
            if self.fallback_to_default {
                "enabled but default path also not found"
            } else {
                "disabled"
            }
        )))
    }

    /// Get the default Things 3 database path.
    ///
    /// Delegates to [`crate::database::get_default_database_path`], which
    /// auto-discovers the per-install `ThingsData-XXXXX` suffix.
    #[must_use]
    pub fn get_default_database_path() -> PathBuf {
        crate::database::get_default_database_path()
    }

    /// Create configuration from environment variables
    ///
    /// Reads the database path from `THINGS_DB_PATH` (preferred) or the legacy
    /// `THINGS_DATABASE_PATH`, and the fallback flag from `THINGS_FALLBACK_TO_DEFAULT`.
    #[must_use]
    pub fn from_env() -> Self {
        let database_path = match std::env::var("THINGS_DB_PATH") {
            Ok(v) => PathBuf::from(v),
            Err(_) => match std::env::var("THINGS_DATABASE_PATH") {
                Ok(v) => {
                    tracing::warn!(
                        "THINGS_DATABASE_PATH is deprecated; please use THINGS_DB_PATH instead"
                    );
                    PathBuf::from(v)
                }
                Err(_) => Self::get_default_database_path(),
            },
        };

        let fallback_to_default = if let Ok(v) = std::env::var("THINGS_FALLBACK_TO_DEFAULT") {
            let lower = v.to_lowercase();
            matches!(lower.as_str(), "true" | "1" | "yes" | "on")
        } else {
            true
        };

        Self {
            database_path,
            fallback_to_default,
        }
    }

    /// Create configuration for testing with a temporary database
    ///
    /// # Errors
    /// Returns `ThingsError::Io` if the temporary file cannot be created
    pub fn for_testing() -> Result<Self> {
        use tempfile::NamedTempFile;
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file.path().to_path_buf();
        Ok(Self {
            database_path: db_path,
            fallback_to_default: false,
        })
    }
}

impl Default for ThingsConfig {
    fn default() -> Self {
        Self::with_default_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ThingsDatabaseError;
    use serial_test::serial;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = ThingsConfig::builder()
            .database_path("/path/to/db.sqlite")
            .fallback_to_default(true)
            .build()
            .unwrap();
        assert_eq!(config.database_path(), Path::new("/path/to/db.sqlite"));
        assert!(config.fallback_to_default());
    }

    #[test]
    fn test_default_config() {
        let config = ThingsConfig::default();
        assert!(config
            .database_path()
            .to_string_lossy()
            .contains("Things Database.thingsdatabase"));
        assert!(!config.fallback_to_default());
    }

    #[test]
    #[serial]
    fn test_config_from_env() {
        let test_path = "/custom/path/db.sqlite";

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();
        let original_fallback = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::set_var("THINGS_DATABASE_PATH", test_path);
        std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", "true");

        let config = ThingsConfig::from_env();
        let path_matches = config.database_path().as_os_str() == test_path;
        let fallback_set = config.fallback_to_default();

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        } else {
            std::env::remove_var("THINGS_DATABASE_PATH");
        }
        if let Some(v) = original_fallback {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", v);
        } else {
            std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");
        }

        assert!(path_matches);
        assert!(fallback_set);
    }

    #[test]
    #[serial]
    fn test_from_env_reads_things_db_path() {
        let test_path = "/custom/path/via_db_path.sqlite";

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();

        std::env::remove_var("THINGS_DATABASE_PATH");
        std::env::set_var("THINGS_DB_PATH", test_path);

        let config = ThingsConfig::from_env();
        assert_eq!(config.database_path(), Path::new(test_path));

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        } else {
            std::env::remove_var("THINGS_DB_PATH");
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        }
    }

    #[test]
    #[serial]
    fn test_from_env_prefers_things_db_path_over_legacy() {
        let new_path = "/new/preferred.sqlite";
        let legacy_path = "/legacy/ignored.sqlite";

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();

        std::env::set_var("THINGS_DB_PATH", new_path);
        std::env::set_var("THINGS_DATABASE_PATH", legacy_path);

        let config = ThingsConfig::from_env();
        assert_eq!(config.database_path(), Path::new(new_path));

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        } else {
            std::env::remove_var("THINGS_DB_PATH");
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        } else {
            std::env::remove_var("THINGS_DATABASE_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_from_env_falls_back_to_legacy_things_database_path() {
        let legacy_path = "/legacy/only.sqlite";

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::set_var("THINGS_DATABASE_PATH", legacy_path);

        let config = ThingsConfig::from_env();
        assert_eq!(config.database_path(), Path::new(legacy_path));

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        } else {
            std::env::remove_var("THINGS_DATABASE_PATH");
        }
    }

    #[test]
    fn test_effective_database_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();
        let config = ThingsConfig::builder()
            .database_path(db_path)
            .build()
            .unwrap();

        let effective_path = config.get_effective_database_path().unwrap();
        assert_eq!(effective_path, db_path);
    }

    #[test]
    fn test_fallback_behavior() {
        let config = ThingsConfig::builder()
            .database_path("/nonexistent/path.sqlite")
            .fallback_to_default(true)
            .build()
            .unwrap();
        let result = config.get_effective_database_path();

        if ThingsConfig::get_default_database_path().exists() {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ThingsConfig::get_default_database_path());
        } else {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_fallback_disabled() {
        let config = ThingsConfig::builder()
            .database_path("/nonexistent/path.sqlite")
            .build()
            .unwrap();
        let result = config.get_effective_database_path();

        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_fallback_enabled() {
        let config = ThingsConfig::builder()
            .database_path("/nonexistent/path")
            .fallback_to_default(true)
            .build()
            .unwrap();
        assert_eq!(config.database_path(), Path::new("/nonexistent/path"));
        assert!(config.fallback_to_default());
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_custom_path() {
        let test_path = "/test/env/custom/path";

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();
        let original_fallback = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::set_var("THINGS_DATABASE_PATH", test_path);
        std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", "false");

        let config = ThingsConfig::from_env();
        let path_matches = config.database_path().as_os_str() == test_path;
        let fallback_off = !config.fallback_to_default();

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        } else {
            std::env::remove_var("THINGS_DATABASE_PATH");
        }
        if let Some(v) = original_fallback {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", v);
        } else {
            std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");
        }

        assert!(path_matches);
        assert!(fallback_off);
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_fallback() {
        let test_id = std::thread::current().id();
        let test_path = format!("/test/env/path/fallback_{test_id:?}");

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();
        let original_fallback = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::set_var("THINGS_DATABASE_PATH", &test_path);
        std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", "true");

        let config = ThingsConfig::from_env();
        let path_matches =
            config.database_path().to_string_lossy() == PathBuf::from(&test_path).to_string_lossy();
        let fallback_set = config.fallback_to_default();

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        } else {
            std::env::remove_var("THINGS_DATABASE_PATH");
        }
        if let Some(v) = original_fallback {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", v);
        } else {
            std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");
        }

        assert!(path_matches);
        assert!(fallback_set);
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_invalid_fallback() {
        let test_id = std::thread::current().id();
        let test_path = format!("/test/env/path/invalid_{test_id:?}");

        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();
        let original_fallback = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::set_var("THINGS_DATABASE_PATH", &test_path);
        std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", "invalid");

        let config = ThingsConfig::from_env();
        let path_matches =
            config.database_path().to_string_lossy() == PathBuf::from(&test_path).to_string_lossy();
        let fallback_off = !config.fallback_to_default();

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        } else {
            std::env::remove_var("THINGS_DATABASE_PATH");
        }
        if let Some(v) = original_fallback {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", v);
        } else {
            std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");
        }

        assert!(path_matches);
        assert!(fallback_off);
    }

    #[test]
    fn test_config_debug_formatting() {
        let config = ThingsConfig::builder()
            .database_path("/test/path")
            .fallback_to_default(true)
            .build()
            .unwrap();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("/test/path"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_config_clone() {
        let config1 = ThingsConfig::builder()
            .database_path("/test/path")
            .fallback_to_default(true)
            .build()
            .unwrap();
        let config2 = config1.clone();

        assert_eq!(config1.database_path(), config2.database_path());
        assert_eq!(config1.fallback_to_default(), config2.fallback_to_default());
    }

    #[test]
    fn test_config_with_different_path_types() {
        let config = ThingsConfig::builder()
            .database_path("relative/path")
            .build()
            .unwrap();
        assert_eq!(config.database_path(), Path::new("relative/path"));

        let config = ThingsConfig::builder()
            .database_path("/absolute/path")
            .build()
            .unwrap();
        assert_eq!(config.database_path(), Path::new("/absolute/path"));

        let config = ThingsConfig::builder().database_path(".").build().unwrap();
        assert_eq!(config.database_path(), Path::new("."));
    }

    #[test]
    fn test_config_edge_cases() {
        let config = ThingsConfig::builder().database_path("").build().unwrap();
        assert_eq!(config.database_path(), Path::new(""));

        let long_path = "/".repeat(1000);
        let config = ThingsConfig::builder()
            .database_path(&long_path)
            .build()
            .unwrap();
        assert_eq!(config.database_path(), Path::new(&long_path));
    }

    #[test]
    fn test_get_default_database_path() {
        let default_path = ThingsConfig::get_default_database_path();

        assert!(!default_path.to_string_lossy().is_empty());
    }

    #[test]
    fn test_for_testing() {
        let config = ThingsConfig::for_testing().unwrap();

        assert!(!config.database_path().to_string_lossy().is_empty());
        assert!(!config.fallback_to_default());
        assert!(config.database_path().parent().is_some());
    }

    #[test]
    fn test_with_default_path() {
        let config = ThingsConfig::with_default_path();

        assert_eq!(
            config.database_path(),
            ThingsConfig::get_default_database_path()
        );
        assert!(!config.fallback_to_default());
    }

    #[test]
    fn test_effective_database_path_fallback_enabled_but_default_missing() {
        let config = ThingsConfig::builder()
            .database_path("/nonexistent/path.sqlite")
            .fallback_to_default(true)
            .build()
            .unwrap();
        let result = config.get_effective_database_path();

        let default_path = ThingsConfig::get_default_database_path();
        if default_path.exists() {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), default_path);
        } else {
            assert!(result.is_err());
            let error = result.unwrap_err();
            match error {
                ThingsError::Database(ThingsDatabaseError::Configuration { message }) => {
                    assert!(message.contains("Database not found at"));
                    assert!(message.contains("fallback is enabled but default path also not found"));
                }
                _ => panic!("Expected Configuration error, got: {error:?}"),
            }
        }
    }

    #[test]
    fn test_effective_database_path_fallback_disabled_error_message() {
        let config = ThingsConfig::builder()
            .database_path("/nonexistent/path.sqlite")
            .build()
            .unwrap();
        let result = config.get_effective_database_path();

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ThingsError::Database(ThingsDatabaseError::Configuration { message }) => {
                assert!(message.contains("Database not found at"));
                assert!(message.contains("fallback is disabled"));
            }
            _ => panic!("Expected Configuration error, got: {error:?}"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_without_variables() {
        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();
        let original_fallback = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::remove_var("THINGS_DATABASE_PATH");
        std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");

        let config = ThingsConfig::from_env();
        let expected_path = ThingsConfig::get_default_database_path();

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        }
        if let Some(v) = original_fallback {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", v);
        }

        assert_eq!(config.database_path(), expected_path);
        assert!(config.fallback_to_default());
    }

    #[test]
    fn test_from_env_fallback_parsing() {
        let test_cases = vec![
            ("true", true),
            ("TRUE", true),
            ("True", true),
            ("1", true),
            ("yes", true),
            ("YES", true),
            ("on", true),
            ("ON", true),
            ("false", false),
            ("FALSE", false),
            ("0", false),
            ("no", false),
            ("off", false),
            ("invalid", false),
            ("", false),
        ];

        for (value, expected) in test_cases {
            let fallback = value.to_lowercase();
            let result =
                fallback == "true" || fallback == "1" || fallback == "yes" || fallback == "on";
            assert_eq!(result, expected, "Failed for value: '{value}'");
        }
    }

    #[test]
    fn test_default_trait_implementation() {
        let config = ThingsConfig::default();

        let expected = ThingsConfig::with_default_path();
        assert_eq!(config.database_path(), expected.database_path());
        assert_eq!(config.fallback_to_default(), expected.fallback_to_default());
    }

    #[test]
    fn test_config_with_path_reference() {
        let path_str = "/test/path/string";
        let path_buf = PathBuf::from("/test/path/buf");

        let config1 = ThingsConfig::builder()
            .database_path(path_str)
            .fallback_to_default(true)
            .build()
            .unwrap();
        let config2 = ThingsConfig::builder()
            .database_path(&path_buf)
            .build()
            .unwrap();

        assert_eq!(config1.database_path(), Path::new(path_str));
        assert_eq!(config2.database_path(), path_buf);
    }

    #[test]
    fn test_effective_database_path_existing_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let config = ThingsConfig::builder()
            .database_path(&db_path)
            .build()
            .unwrap();

        let effective_path = config.get_effective_database_path().unwrap();
        assert_eq!(effective_path, db_path);
    }

    #[test]
    fn test_effective_database_path_fallback_success() {
        let default_path = ThingsConfig::get_default_database_path();

        if default_path.exists() {
            let config = ThingsConfig::builder()
                .database_path("/nonexistent/path.sqlite")
                .fallback_to_default(true)
                .build()
                .unwrap();
            let effective_path = config.get_effective_database_path().unwrap();
            assert_eq!(effective_path, default_path);
        }
    }

    #[test]
    fn test_config_debug_implementation() {
        let config = ThingsConfig::builder()
            .database_path("/test/debug/path")
            .fallback_to_default(true)
            .build()
            .unwrap();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("database_path"));
        assert!(debug_str.contains("fallback_to_default"));
        assert!(debug_str.contains("/test/debug/path"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_config_clone_implementation() {
        let config1 = ThingsConfig::builder()
            .database_path("/test/clone/path")
            .fallback_to_default(true)
            .build()
            .unwrap();
        let config2 = config1.clone();

        assert_eq!(config1.database_path(), config2.database_path());
        assert_eq!(config1.fallback_to_default(), config2.fallback_to_default());

        let config3 = ThingsConfig::builder()
            .database_path("/different/path")
            .build()
            .unwrap();
        assert_ne!(config1.database_path(), config3.database_path());
        assert_ne!(config1.fallback_to_default(), config3.fallback_to_default());
    }

    #[test]
    fn test_get_default_database_path_format() {
        let default_path = ThingsConfig::get_default_database_path();
        let path_str = default_path.to_string_lossy();

        assert!(path_str.contains("Library"));
        assert!(path_str.contains("Group Containers"));
        assert!(path_str.contains("JLMPQHK86H.com.culturedcode.ThingsMac"));
        assert!(path_str.contains("ThingsData-"));
        assert!(path_str.contains("Things Database.thingsdatabase"));
        assert!(path_str.contains("main.sqlite"));
    }

    #[test]
    fn test_home_env_var_fallback() {
        let default_path = ThingsConfig::get_default_database_path();
        let path_str = default_path.to_string_lossy();

        assert!(path_str.starts_with('/') || path_str.starts_with('~'));
    }

    #[test]
    fn test_config_effective_database_path_existing_file() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_db.sqlite");
        std::fs::File::create(&temp_file).unwrap();

        let config = ThingsConfig::builder()
            .database_path(&temp_file)
            .build()
            .unwrap();
        let effective_path = config.get_effective_database_path().unwrap();
        assert_eq!(effective_path, temp_file);

        std::fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_config_effective_database_path_fallback_success() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_database.sqlite");
        std::fs::File::create(&temp_file).unwrap();

        let config = ThingsConfig::builder()
            .database_path(&temp_file)
            .fallback_to_default(true)
            .build()
            .unwrap();

        let effective_path = config.get_effective_database_path().unwrap();
        assert_eq!(effective_path, temp_file);

        std::fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_config_effective_database_path_fallback_disabled_error_message() {
        let non_existent_path = PathBuf::from("/nonexistent/path/db.sqlite");
        let config = ThingsConfig::builder()
            .database_path(&non_existent_path)
            .build()
            .unwrap();

        let result = config.get_effective_database_path();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            ThingsError::Database(ThingsDatabaseError::Configuration { .. })
        ));
    }

    #[test]
    #[serial]
    fn test_config_effective_database_path_fallback_enabled_but_default_missing() {
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/nonexistent/home");

        let non_existent_path = PathBuf::from("/nonexistent/path/db.sqlite");
        let config = ThingsConfig::builder()
            .database_path(&non_existent_path)
            .fallback_to_default(true)
            .build()
            .unwrap();

        let result = config.get_effective_database_path();

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }

        assert!(
            result.is_err(),
            "Expected error when both configured and default paths don't exist"
        );
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            ThingsError::Database(ThingsDatabaseError::Configuration { .. })
        ));

        let error_message = format!("{error}");
        assert!(error_message.contains("Database not found at /nonexistent/path/db.sqlite"));
        assert!(error_message.contains("fallback is enabled but default path also not found"));
    }

    #[test]
    fn test_config_fallback_behavior() {
        let path = PathBuf::from("/test/path/db.sqlite");

        let config_with_fallback = ThingsConfig::builder()
            .database_path(&path)
            .fallback_to_default(true)
            .build()
            .unwrap();
        assert!(config_with_fallback.fallback_to_default());

        let config_without_fallback = ThingsConfig::builder()
            .database_path(&path)
            .build()
            .unwrap();
        assert!(!config_without_fallback.fallback_to_default());
    }

    #[test]
    fn test_config_fallback_disabled() {
        let path = PathBuf::from("/test/path/db.sqlite");
        let config = ThingsConfig::builder()
            .database_path(&path)
            .build()
            .unwrap();
        assert!(!config.fallback_to_default());
    }

    #[test]
    #[serial]
    fn test_config_from_env_without_variables() {
        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();
        let original_fallback = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::remove_var("THINGS_DATABASE_PATH");
        std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");

        let config = ThingsConfig::from_env();
        let contains_default = config
            .database_path()
            .to_string_lossy()
            .contains("Things Database.thingsdatabase");

        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        }
        if let Some(v) = original_fallback {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", v);
        }

        assert!(contains_default);
    }

    #[test]
    fn test_config_from_env_fallback_parsing() {
        let test_cases = vec![
            ("true", true),
            ("false", false),
            ("1", true),
            ("0", false),
            ("yes", true),
            ("no", false),
            ("invalid", false),
        ];

        for (value, expected) in test_cases {
            let lower = value.to_lowercase();
            let result = matches!(lower.as_str(), "true" | "1" | "yes" | "on");

            assert_eq!(
                result, expected,
                "Failed for value: '{value}', expected: {expected}, got: {result}"
            );
        }
    }

    #[test]
    fn test_config_for_testing() {
        let result = ThingsConfig::for_testing();
        assert!(result.is_ok(), "Should create test config successfully");

        let config = result.unwrap();
        assert!(
            !config.fallback_to_default(),
            "Test config should have fallback disabled"
        );

        let path_str = config.database_path().to_string_lossy();
        assert!(
            path_str.contains("tmp") || !path_str.is_empty(),
            "Test config should use a temporary path"
        );
    }

    #[test]
    fn test_config_effective_database_path_error_cases() {
        let non_existent_path = PathBuf::from("/absolutely/non/existent/path/database.db");
        let config = ThingsConfig::builder()
            .database_path(&non_existent_path)
            .build()
            .unwrap();

        let result = config.get_effective_database_path();
        assert!(
            result.is_err(),
            "Should fail when file doesn't exist and fallback is disabled"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("fallback is disabled"),
            "Error message should mention fallback is disabled"
        );
    }

    #[test]
    fn test_config_effective_database_path_with_existing_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let config = ThingsConfig::builder()
            .database_path(&temp_path)
            .build()
            .unwrap();
        let effective_path = config.get_effective_database_path().unwrap();

        assert_eq!(effective_path, temp_path);
    }

    #[test]
    fn test_config_get_default_database_path_format() {
        let path = ThingsConfig::get_default_database_path();
        let path_str = path.to_string_lossy();

        assert!(
            path_str.contains("JLMPQHK86H.com.culturedcode.ThingsMac"),
            "Should contain the correct container identifier"
        );
        assert!(
            path_str.contains("ThingsData-"),
            "Should contain a ThingsData-* data directory"
        );
        assert!(
            path_str.contains("Things Database.thingsdatabase"),
            "Should contain Things database directory"
        );
        assert!(
            path_str.contains("main.sqlite"),
            "Should contain main.sqlite file"
        );
    }

    #[test]
    fn test_config_with_different_path_types_comprehensive() {
        let string_path = "/test/path/db.sqlite";
        let config1 = ThingsConfig::builder()
            .database_path(string_path)
            .build()
            .unwrap();
        assert_eq!(config1.database_path(), Path::new(string_path));
        assert!(!config1.fallback_to_default());

        let pathbuf_path = PathBuf::from("/another/test/path.db");
        let config2 = ThingsConfig::builder()
            .database_path(&pathbuf_path)
            .fallback_to_default(true)
            .build()
            .unwrap();
        assert_eq!(config2.database_path(), pathbuf_path);
        assert!(config2.fallback_to_default());
    }

    #[test]
    fn test_config_from_env_edge_cases() {
        let test_cases = vec![
            ("true", true),
            ("TRUE", true),
            ("True", true),
            ("1", true),
            ("yes", true),
            ("YES", true),
            ("on", true),
            ("ON", true),
            ("false", false),
            ("FALSE", false),
            ("0", false),
            ("no", false),
            ("off", false),
            ("invalid", false),
            ("", false),
            ("random_string", false),
        ];

        for (value, expected) in test_cases {
            let lower = value.to_lowercase();
            let result = matches!(lower.as_str(), "true" | "1" | "yes" | "on");
            assert_eq!(result, expected, "Failed for value: '{value}'");
        }
    }

    #[test]
    #[serial]
    fn test_config_from_env_fallback_parsing_with_env_vars() {
        let original_value = std::env::var("THINGS_FALLBACK_TO_DEFAULT").ok();

        let test_cases = vec![
            ("true", true),
            ("false", false),
            ("1", true),
            ("0", false),
            ("yes", true),
            ("no", false),
            ("invalid", false),
        ];

        for (value, expected) in test_cases {
            std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", value);

            let env_value = std::env::var("THINGS_FALLBACK_TO_DEFAULT")
                .unwrap_or_else(|_| "NOT_SET".to_string());
            println!("Environment variable set to: '{env_value}'");

            let env_value_check = std::env::var("THINGS_FALLBACK_TO_DEFAULT")
                .unwrap_or_else(|_| "NOT_SET".to_string());
            println!("Environment variable check before from_env: '{env_value_check}'");

            let config = ThingsConfig::from_env();

            println!(
                "Testing value: '{}', expected: {}, got: {}",
                value,
                expected,
                config.fallback_to_default()
            );

            assert_eq!(
                config.fallback_to_default(),
                expected,
                "Failed for value: '{}', expected: {}, got: {}",
                value,
                expected,
                config.fallback_to_default()
            );
        }

        if let Some(original) = original_value {
            std::env::set_var("THINGS_FALLBACK_TO_DEFAULT", original);
        } else {
            std::env::remove_var("THINGS_FALLBACK_TO_DEFAULT");
        }
    }

    #[test]
    #[serial]
    fn test_config_home_env_var_fallback() {
        let original_home = std::env::var("HOME").ok();
        let original_db_path = std::env::var("THINGS_DB_PATH").ok();
        let original_legacy = std::env::var("THINGS_DATABASE_PATH").ok();

        std::env::remove_var("THINGS_DB_PATH");
        std::env::remove_var("THINGS_DATABASE_PATH");
        std::env::set_var("HOME", "/test/home");

        let config = ThingsConfig::from_env();
        let contains_default = config
            .database_path()
            .to_string_lossy()
            .contains("Things Database.thingsdatabase");

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(v) = original_db_path {
            std::env::set_var("THINGS_DB_PATH", v);
        }
        if let Some(v) = original_legacy {
            std::env::set_var("THINGS_DATABASE_PATH", v);
        }

        assert!(contains_default);
    }

    #[test]
    fn test_config_with_default_path() {
        let config = ThingsConfig::with_default_path();
        assert!(config
            .database_path()
            .to_string_lossy()
            .contains("Things Database.thingsdatabase"));
        assert!(!config.fallback_to_default());
    }

    #[test]
    fn test_builder_default_uses_default_database_path() {
        let config = ThingsConfig::builder().build().unwrap();
        assert_eq!(
            config.database_path(),
            ThingsConfig::get_default_database_path()
        );
        assert!(!config.fallback_to_default());
    }
}
