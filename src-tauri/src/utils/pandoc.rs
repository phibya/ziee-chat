use std::path::PathBuf;
use std::process::Command;

use crate::utils::resource_paths::ResourcePaths;

/// Pandoc utility functions for document processing
pub struct PandocUtils;

impl PandocUtils {
    /// Check if Pandoc is available and get its path
    ///
    /// This function first tries to find a bundled Pandoc binary using ResourcePaths,
    /// then falls back to the system Pandoc installation.
    ///
    /// # Returns
    /// - `Some(PathBuf)` if Pandoc is found and working
    /// - `None` if Pandoc is not available
    pub fn get_pandoc_path() -> Option<PathBuf> {
        // First try to find bundled Pandoc binary
        if let Some(pandoc_path) = ResourcePaths::find_executable_binary("pandoc") {
            // Test if the bundled binary works
            let test_result = Command::new(&pandoc_path).arg("--version").output();

            if let Ok(output) = test_result {
                if output.status.success() {
                    return Some(pandoc_path);
                }
            }
        }

        // Fallback to system Pandoc
        let system_test = Command::new("pandoc").arg("--version").output();

        match system_test {
            Ok(output) if output.status.success() => Some(PathBuf::from("pandoc")),
            _ => None,
        }
    }

    /// Check if Pandoc is available
    ///
    /// # Returns
    /// - `true` if Pandoc is available and working
    /// - `false` if Pandoc is not available
    pub fn is_pandoc_available() -> bool {
        Self::get_pandoc_path().is_some()
    }

    /// Get Pandoc version string
    ///
    /// # Returns
    /// - `Some(String)` containing the version if Pandoc is available
    /// - `None` if Pandoc is not available or version check fails
    pub fn get_pandoc_version() -> Option<String> {
        let pandoc_path = Self::get_pandoc_path()?;

        let output = Command::new(&pandoc_path).arg("--version").output().ok()?;

        if output.status.success() {
            let version_output = String::from_utf8(output.stdout).ok()?;
            // Extract just the first line which contains the version
            version_output.lines().next().map(|s| s.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pandoc_path_detection() {
        // This test will vary based on whether Pandoc is installed
        let path = PandocUtils::get_pandoc_path();

        if let Some(pandoc_path) = path {
            // If Pandoc is found, it should be a valid path
            assert!(!pandoc_path.as_os_str().is_empty());
        }

        // The availability check should match the path detection
        assert_eq!(
            PandocUtils::is_pandoc_available(),
            PandocUtils::get_pandoc_path().is_some()
        );
    }

    #[test]
    fn test_pandoc_version() {
        if PandocUtils::is_pandoc_available() {
            let version = PandocUtils::get_pandoc_version();
            assert!(version.is_some());

            if let Some(v) = version {
                // Version string should contain "pandoc"
                assert!(v.to_lowercase().contains("pandoc"));
            }
        }
    }
}
