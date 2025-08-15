use std::path::PathBuf;

/// Resource path resolution for different platforms
///
/// This utility provides platform-specific path resolution for bundled resources
/// like hub files, libraries, and other assets that need different locations
/// in development vs production environments.
pub struct ResourcePaths;

impl ResourcePaths {
    /// Get platform-specific resource folder paths with fallback logic
    ///
    /// # Arguments
    /// * `resource_name` - The name of the resource folder (e.g., "hub", "lib")
    ///
    /// # Returns
    /// Vector of paths to try in order of preference:
    /// - Development: ./resource_name or ../resource_name (from CARGO_MANIFEST_DIR)
    /// - Production (Windows/Linux): ./resource_name
    /// - Production (macOS): ../Resources/resource_name then ./resource_name
    pub fn get_resource_paths(resource_name: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Development environment check
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let dev_path = PathBuf::from(manifest_dir)
                .parent()
                .unwrap()
                .join(resource_name);
            paths.push(dev_path);
        }

        // Get executable directory for relative paths
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Development: Check same directory as executable
        paths.push(exe_dir.join(resource_name));

        // Production: Platform-specific paths
        if cfg!(target_os = "macos") {
            // macOS app bundle: ../Resources/resource_name
            paths.push(exe_dir.join("../Resources").join(resource_name));
        }

        // Fallback: current directory
        paths.push(PathBuf::from(resource_name));

        paths
    }

    /// Find the first existing resource folder from the platform-specific paths
    ///
    /// # Arguments
    /// * `resource_name` - The name of the resource folder
    ///
    /// # Returns
    /// The first existing path, or the first path if none exist (for creation)
    pub fn find_resource_folder(resource_name: &str) -> PathBuf {
        let paths = Self::get_resource_paths(resource_name);

        for path in &paths {
            if path.exists() {
                return path.clone();
            }
        }

        // Return first path if none exist (for creation)
        paths
            .into_iter()
            .next()
            .unwrap_or_else(|| PathBuf::from(resource_name))
    }

    /// Get platform-specific library paths for dynamic library loading
    ///
    /// # Arguments
    /// * `library_name` - The name of the library file (e.g., "libpdfium.dylib")
    ///
    /// # Returns
    /// Vector of absolute library search paths in order of preference
    pub fn get_library_search_paths(library_name: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // Get the directory of the current executable for absolute paths
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Platform-specific library search paths
        if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
            // Windows and Linux: ./lib/
            if let Ok(lib_path) = exe_dir.join("lib").canonicalize() {
                paths.push(lib_path.to_string_lossy().to_string());
            }
            if let Ok(exe_path) = exe_dir.canonicalize() {
                paths.push(exe_path.to_string_lossy().to_string());
            }
        } else if cfg!(target_os = "macos") {
            // macOS: ./lib/ (development) and ../Resources/lib/ (production)
            if let Ok(lib_path) = exe_dir.join("lib").canonicalize() {
                paths.push(lib_path.to_string_lossy().to_string());
            }
            if let Ok(resources_lib_path) = exe_dir.join("../Resources/lib").canonicalize() {
                paths.push(resources_lib_path.to_string_lossy().to_string());
            }
            if let Ok(exe_path) = exe_dir.canonicalize() {
                paths.push(exe_path.to_string_lossy().to_string());
            }
        } else {
            // Default: current directory
            if let Ok(exe_path) = exe_dir.canonicalize() {
                paths.push(exe_path.to_string_lossy().to_string());
            }
        }

        // Also add specific library name paths
        if let Ok(lib_file_path) = exe_dir.join(library_name).canonicalize() {
            paths.push(lib_file_path.to_string_lossy().to_string());
        }

        paths
    }

    /// Get hub folder path (convenience method for hub-specific logic)
    pub fn get_hub_folder() -> PathBuf {
        Self::find_resource_folder("hub")
    }

    /// Get executable binary path in the same folder as the current running executable
    ///
    /// # Arguments
    /// * `binary_name` - The name of the binary without extension (e.g., "pandoc")
    ///
    /// # Returns
    /// The full path to the binary, with .exe extension added on Windows
    pub fn get_executable_binary_path(binary_name: &str) -> PathBuf {
        // Get the directory of the current executable
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Add .exe extension on Windows
        let binary_filename = if cfg!(target_os = "windows") {
            format!("{}.exe", binary_name)
        } else {
            binary_name.to_string()
        };

        exe_dir.join(binary_filename)
    }

    /// Get executable binary path with fallback search in bin/ subdirectory
    ///
    /// # Arguments
    /// * `binary_name` - The name of the binary without extension (e.g., "pandoc")
    ///
    /// # Returns
    /// The full path to the binary, searching current directory first, then bin/ subdirectory
    pub fn find_executable_binary(binary_name: &str) -> Option<PathBuf> {
        // First try in the same directory as the current executable
        let primary_path = Self::get_executable_binary_path(binary_name);
        if primary_path.exists() {
            return Some(primary_path);
        }

        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        let binary_filename = if cfg!(target_os = "windows") {
            format!("{}.exe", binary_name)
        } else {
            binary_name.to_string()
        };

        #[cfg(target_os = "macos")]
        {
            // On macOS, check for production bundle first (Resources/bin)
            let resources_bin_path = exe_dir.join("../Resources/bin").join(&binary_filename);
            if resources_bin_path.exists() {
                return Some(resources_bin_path);
            }

            // Then check development location (bin)
            let dev_bin_path = exe_dir.join("bin").join(&binary_filename);
            if dev_bin_path.exists() {
                return Some(dev_bin_path);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On other platforms, try bin/ subdirectory
            let bin_path = exe_dir.join("bin").join(&binary_filename);
            if bin_path.exists() {
                return Some(bin_path);
            }
        }

        None
    }
}
