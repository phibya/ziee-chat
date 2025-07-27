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
            let dev_path = PathBuf::from(manifest_dir).parent().unwrap().join(resource_name);
            paths.push(dev_path);
        }
        
        // Get executable directory for relative paths
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        
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
        paths.into_iter().next().unwrap_or_else(|| PathBuf::from(resource_name))
    }
    
    /// Get platform-specific library paths for dynamic library loading
    /// 
    /// # Arguments
    /// * `library_name` - The name of the library file (e.g., "libpdfium.dylib")
    /// 
    /// # Returns
    /// Vector of library search paths in order of preference
    pub fn get_library_search_paths(library_name: &str) -> Vec<String> {
        let mut paths = Vec::new();
        
        // Platform-specific library search paths
        if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
            // Windows and Linux: ./lib/
            paths.push("./lib/".to_string());
            paths.push("./".to_string());
        } else if cfg!(target_os = "macos") {
            // macOS: ./lib/ (development) and ../Resources/lib/ (production)
            paths.push("./lib/".to_string());
            paths.push("../Resources/lib/".to_string());
            paths.push("./".to_string());
        } else {
            // Default: current directory
            paths.push("./".to_string());
        }
        
        // Also add specific library name paths
        paths.push(format!("./{}", library_name));
        
        paths
    }
    
    /// Get hub folder path (convenience method for hub-specific logic)
    pub fn get_hub_folder() -> PathBuf {
        Self::find_resource_folder("hub")
    }
    
    /// Get lib folder path (convenience method for library-specific logic)
    pub fn get_lib_folder() -> PathBuf {
        Self::find_resource_folder("lib")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_paths_generation() {
        let paths = ResourcePaths::get_resource_paths("test");
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("test")));
    }
    
    #[test]
    fn test_library_search_paths() {
        let paths = ResourcePaths::get_library_search_paths("libtest.dylib");
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.contains("lib/")));
    }
}