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
}
