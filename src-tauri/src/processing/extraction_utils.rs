use std::path::Path;
use std::process::Command;
use base64::Engine;
use crate::database::queries::document_extraction;
use crate::database::models::document_extraction::*;
use crate::utils::resource_paths::ResourcePaths;

// Simple PDF text extraction using pdf-extract crate (if available) or pdfinfo fallback
pub async fn extract_pdf_simple(file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // For now, return a placeholder - we can implement pdf-extract later
    let filename = file_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.pdf");
    
    Ok(format!("[PDF file: {}]\n\nSimple PDF text extraction is not yet implemented. Please use OCR or LLM extraction methods.", filename))
}

// OCR extraction using tesseract
pub async fn extract_pdf_ocr(file_path: &Path, settings: &OcrExtractionSettings) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // First convert PDF to images using pdfium (reuse existing thumbnail code)
    let images = pdf_to_images(file_path).await?;
    
    // Then run OCR on each image
    let mut extracted_text = String::new();
    
    for (page_num, image_path) in images.iter().enumerate() {
        let page_text = run_tesseract_ocr(image_path, &settings.language).await?;
        extracted_text.push_str(&format!("\n--- Page {} ---\n", page_num + 1));
        extracted_text.push_str(&page_text);
        extracted_text.push('\n');
    }
    
    Ok(extracted_text)
}

// LLM extraction using vision models
pub async fn extract_pdf_llm(file_path: &Path, settings: &LlmExtractionSettings) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Convert PDF to images
    let images = pdf_to_images(file_path).await?;
    
    // Send images to LLM for text extraction
    let mut extracted_text = String::new();
    
    for (page_num, image_path) in images.iter().enumerate() {
        let page_text = call_llm_vision_api(image_path, settings).await?;
        extracted_text.push_str(&format!("\n--- Page {} ---\n", page_num + 1));
        extracted_text.push_str(&page_text);
        extracted_text.push('\n');
    }
    
    Ok(extracted_text)
}

// Image OCR extraction
pub async fn extract_image_ocr(file_path: &Path, settings: &OcrExtractionSettings) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    run_tesseract_ocr(file_path, &settings.language).await
}

// Image LLM extraction  
pub async fn extract_image_llm(file_path: &Path, settings: &LlmExtractionSettings) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    call_llm_vision_api(file_path, settings).await
}

// Helper function to convert PDF to images (reuse pdfium from thumbnails)
async fn pdf_to_images(file_path: &Path) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    use pdfium_render::prelude::*;
    use image::{RgbImage, ImageBuffer};
    use crate::utils::resource_paths::ResourcePaths;
    
    // Initialize PDFium (same as thumbnail generator)
    let pdfium = {
        let library_name = if cfg!(target_os = "windows") {
            "pdfium.dll"
        } else if cfg!(target_os = "macos") {
            "libpdfium.dylib"
        } else {
            "libpdfium.so"
        };
        
        let search_paths = ResourcePaths::get_library_search_paths(library_name);
        
        let mut pdfium_result = None;
        for path in &search_paths {
            if let Ok(lib) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path)) {
                pdfium_result = Some(lib);
                break;
            }
        }
        
        pdfium_result
            .or_else(|| Pdfium::bind_to_system_library().ok())
            .ok_or(format!("Failed to load PDFium library. Searched paths: {:?}", search_paths))?  
    };

    let pdfium = Pdfium::new(pdfium);

    // Load the PDF document
    let document = pdfium.load_pdf_from_file(file_path, None)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;

    let page_count = document.pages().len() as u32;
    let mut image_paths = Vec::new();
    
    // Create temp directory for images
    let temp_dir = std::env::temp_dir().join("pdf_extraction");
    std::fs::create_dir_all(&temp_dir)?;

    // Generate full-resolution images for each page
    for page_index in 0..page_count {
        let page = document.pages().get(page_index as u16)
            .map_err(|e| format!("Failed to get page {}: {}", page_index + 1, e))?;

        // Render page to bitmap with higher resolution for text extraction
        let render_config = PdfRenderConfig::new()
            .set_target_width(2048)  // Higher resolution for better OCR
            .set_maximum_height(2048)
            .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

        let bitmap = page.render_with_config(&render_config)
            .map_err(|e| format!("Failed to render page {}: {}", page_index + 1, e))?;

        // Convert bitmap to RGB image
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        
        let pixel_data = bitmap.as_raw_bytes();
        
        // Convert BGRA to RGB
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        for pixel in pixel_data.chunks_exact(4) {
            rgb_data.push(pixel[2]); // R (from B in BGRA)
            rgb_data.push(pixel[1]); // G
            rgb_data.push(pixel[0]); // B (from R in BGRA)
        }

        let rgb_image: RgbImage = ImageBuffer::from_raw(width, height, rgb_data)
            .ok_or("Failed to create RGB image from raw data")?;

        // Save image
        let image_path = temp_dir.join(format!("page_{}.png", page_index + 1));
        rgb_image.save(&image_path)
            .map_err(|e| format!("Failed to save image: {}", e))?;
            
        image_paths.push(image_path);
    }

    Ok(image_paths)
}

// Helper function to run Tesseract OCR
async fn run_tesseract_ocr(image_path: &Path, language: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Get the path to the bundled Tesseract binary
    let tesseract_path = ResourcePaths::find_executable_binary("tesseract")
        .ok_or("Tesseract binary not found. Make sure Tesseract is built and bundled with the application.")?;

    let output = Command::new(&tesseract_path)
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(language)
        .output()?;

    if !output.status.success() {
        return Err(format!("Tesseract OCR failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let text = String::from_utf8(output.stdout)?;
    Ok(text)
}

// Helper function to call LLM vision API
async fn call_llm_vision_api(image_path: &Path, settings: &LlmExtractionSettings) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Read image and convert to base64
    let image_data = std::fs::read(image_path)?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
    
    // For now, return a placeholder - this will need to be implemented with actual model communication
    let filename = image_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    
    Ok(format!("[LLM Vision Extraction]\nModel: {}\nPrompt: {}\nImage: {}\n\nLLM vision extraction is not yet implemented. This would send the image to the vision model for text extraction.", 
        settings.model_id.as_deref().unwrap_or("not configured"), settings.system_prompt, filename))
}

// Main extraction function that uses configuration
pub async fn extract_text_with_config(file_path: &Path, file_type: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let settings = document_extraction::get_current_extraction_settings(file_type).await?;
    
    match settings.method.as_str() {
        "simple" => {
            match file_type {
                "pdf" => {
                    let text = extract_pdf_simple(file_path).await?;
                    Ok(Some(text))
                }
                _ => Ok(None), // Simple extraction only for PDF
            }
        }
        "ocr" => {
            let text = match file_type {
                "pdf" => extract_pdf_ocr(file_path, &settings.ocr).await?,
                "image" => extract_image_ocr(file_path, &settings.ocr).await?,
                _ => return Ok(None),
            };
            Ok(Some(text))
        }
        "llm" => {
            let text = match file_type {
                "pdf" => extract_pdf_llm(file_path, &settings.llm).await?,
                "image" => extract_image_llm(file_path, &settings.llm).await?,
                _ => return Ok(None),
            };
            Ok(Some(text))
        }
        _ => Ok(None), // Unknown method
    }
}