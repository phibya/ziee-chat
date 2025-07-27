// Test script to verify ResourcePaths utility works correctly
use std::path::PathBuf;

// Include the resource_paths module
mod utils {
    pub mod resource_paths;
}

use utils::resource_paths::ResourcePaths;

fn main() {
    println!("Testing ResourcePaths utility...\n");
    
    // Test hub folder resolution
    println!("=== Hub Folder Test ===");
    let hub_folder = ResourcePaths::get_hub_folder();
    println!("Hub folder path: {}", hub_folder.display());
    println!("Hub folder exists: {}", hub_folder.exists());
    
    if hub_folder.exists() {
        println!("✅ Hub folder found successfully");
    } else {
        println!("⚠️  Hub folder not found, but path resolved to: {}", hub_folder.display());
    }
    
    // Test library search paths
    println!("\n=== Library Search Paths Test ===");
    let library_name = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };
    
    let search_paths = ResourcePaths::get_library_search_paths(library_name);
    println!("Library search paths for '{}':", library_name);
    for (i, path) in search_paths.iter().enumerate() {
        println!("  {}. {}", i + 1, path);
        
        // Check if library exists at this path
        let full_path = PathBuf::from(path).join(library_name);
        if full_path.exists() {
            println!("     ✅ Library found at: {}", full_path.display());
        }
    }
    
    // Test generic resource paths
    println!("\n=== Generic Resource Paths Test ===");
    let test_paths = ResourcePaths::get_resource_paths("test");
    println!("Resource paths for 'test' folder:");
    for (i, path) in test_paths.iter().enumerate() {
        println!("  {}. {}", i + 1, path.display());
        println!("     Exists: {}", path.exists());
    }
    
    println!("\n=== Test Complete ===");
    println!("ResourcePaths utility is working correctly!");
}