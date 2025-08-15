use crate::utils::resource_paths::ResourcePaths;
use pdfium_render::prelude::*;

/// Initialize PDFium library using platform-specific paths with fallback to system library
///
/// This function handles the platform-specific initialization of PDFium by:
/// 1. Determining the correct library name for the current platform
/// 2. Searching for the library in bundled resource paths
/// 3. Falling back to system library if bundled library is not found
///
/// # Returns
///
/// Returns a `Result<Box<dyn PdfiumLibraryBindings>, String>` where:
/// - `Ok(Box<dyn PdfiumLibraryBindings>)` contains the initialized PDFium bindings
/// - `Err(String)` contains an error message with details about the search paths
///
/// # Example
///
/// ```rust
/// use crate::utils::pdfium::initialize_pdfium;
/// use pdfium_render::prelude::*;
///
/// match initialize_pdfium() {
///     Ok(bindings) => {
///         let pdfium = Pdfium::new(bindings);
///         // Use pdfium for PDF operations
///     }
///     Err(error) => {
///         eprintln!("Failed to initialize PDFium: {}", error);
///     }
/// }
/// ```
pub fn initialize_pdfium() -> Result<Box<dyn PdfiumLibraryBindings>, String> {
    // Get platform-specific library name
    let library_name = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };

    // Get search paths for the library
    let search_paths = ResourcePaths::get_library_search_paths(library_name);

    // Try each path in order
    let mut pdfium_result = None;
    for path in &search_paths {
        if let Ok(lib) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path))
        {
            pdfium_result = Some(lib);
            break;
        }
    }

    // Return bundled library or fallback to system library
    pdfium_result
        .or_else(|| Pdfium::bind_to_system_library().ok())
        .ok_or_else(|| {
            format!(
                "Failed to load PDFium library. Searched paths: {:?}. Make sure PDFium is installed or the binary is available.",
                search_paths
            )
        })
}
