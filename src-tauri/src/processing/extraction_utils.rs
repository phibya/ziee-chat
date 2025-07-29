use std::path::Path;
use std::process::Command;
use base64::Engine;
use crate::database::queries::document_extraction;
use crate::database::models::document_extraction::*;
use crate::utils::resource_paths::ResourcePaths;
use crate::utils::pdfium::initialize_pdfium;

// Simple PDF text extraction using pdf-extract crate
pub async fn extract_pdf_simple(file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use pdf_extract;
    use std::fs;
    
    // Read the PDF file into bytes
    let pdf_bytes = tokio::task::spawn_blocking({
        let file_path = file_path.to_owned();
        move || fs::read(&file_path)
    }).await??;
    
    // Extract text from PDF bytes using pdf-extract
    let extracted_text = tokio::task::spawn_blocking(move || {
        pdf_extract::extract_text_from_mem(&pdf_bytes)
    }).await??;
    
    // Clean up the extracted text
    let cleaned_text = clean_extracted_text(&extracted_text);
    
    if cleaned_text.trim().is_empty() {
        let filename = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown.pdf");
        return Ok(format!("[PDF file: {}]\n\nNo text content found in PDF. The PDF may contain only images or have text in a format that cannot be extracted. Please try OCR or LLM extraction methods.", filename));
    }
    
    Ok(cleaned_text)
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
    
    // Initialize PDFium using the centralized utility
    let pdfium_bindings = initialize_pdfium()
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error + Send + Sync>)?;

    let pdfium = Pdfium::new(pdfium_bindings);

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
    use crate::ai::{ChatMessage, ChatRequest};
    use crate::database::queries::{models::get_model_by_id, providers::get_provider_by_id};
    use uuid::Uuid;
    
    // Validate model_id
    let model_id_str = settings.model_id.as_ref()
        .ok_or("No model ID specified for LLM vision extraction")?;
    let model_id = Uuid::parse_str(model_id_str)
        .map_err(|_| "Invalid model ID format")?;
    
    // Get model information
    let model = get_model_by_id(model_id).await?
        .ok_or("Model not found")?;
    
    // Get provider information
    let provider = get_provider_by_id(model.provider_id).await?
        .ok_or("Provider not found")?;
    
    // Check if model supports vision
    if !model.capabilities.as_ref()
        .map(|caps| caps.vision.unwrap_or(false))
        .unwrap_or(false) {
        return Err("Selected model does not support vision capabilities".into());
    }
    
    // Read image and convert to base64
    let image_data = std::fs::read(image_path)?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
    
    // Create AI provider
    let ai_provider = crate::api::chat::create_ai_provider_with_model_id(&provider, Some(model_id)).await?;
    
    // Create vision-compatible content
    // For OpenAI-compatible APIs, we'll create a message with image content
    let vision_content = if provider.provider_type == "openai" || provider.provider_type == "groq" {
        // Use OpenAI vision format with base64 image
        format!("data:image/jpeg;base64,{}", base64_image)
    } else {
        // For other providers, embed image description in text
        format!("[Image data: base64 encoded image]\n\n{}", settings.system_prompt)
    };
    
    // Create messages for vision extraction
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: settings.system_prompt.clone(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: vision_content,
        },
    ];
    
    // Create chat request with document extraction parameters
    let chat_request = ChatRequest {
        messages,
        model: model.name.clone(),
        stream: false, // Use non-streaming for document extraction
        temperature: Some(settings.parameters.temperature.unwrap_or(0.2)),
        max_tokens: Some(2048), // Use fixed max tokens for document extraction
        top_p: Some(settings.parameters.top_p.unwrap_or(0.9)),
        frequency_penalty: settings.parameters.frequency_penalty,
        presence_penalty: settings.parameters.presence_penalty,
    };
    
    // Call the AI provider
    match ai_provider.chat(chat_request).await {
        Ok(response) => {
            let extracted_text = response.content.trim();
            
            if extracted_text.is_empty() {
                let filename = image_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");
                return Ok(format!("[Image: {}]\n\nNo text was extracted by the vision model. The image may not contain readable text.", filename));
            }
            
            Ok(extracted_text.to_string())
        }
        Err(e) => {
            eprintln!("LLM vision extraction failed: {}", e);
            Err(format!("LLM vision extraction failed: {}", e).into())
        }
    }
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

/// Clean up extracted text by removing excessive whitespace and normalizing line breaks
fn clean_extracted_text(text: &str) -> String {
    use std::collections::HashSet;
    
    let lines: Vec<&str> = text.lines().collect();
    let mut cleaned_lines = Vec::new();
    let mut seen_lines = HashSet::new();
    
    for line in lines {
        let trimmed = line.trim();
        
        // Skip empty lines and very short lines that are likely artifacts
        if trimmed.is_empty() || trimmed.len() < 2 {
            continue;
        }
        
        // Skip duplicate lines (common in PDFs with headers/footers)
        if seen_lines.contains(trimmed) {
            continue;
        }
        
        seen_lines.insert(trimmed.to_string());
        cleaned_lines.push(trimmed);
    }
    
    // Join lines with proper spacing
    let result = cleaned_lines.join("\n");
    
    // Remove excessive whitespace
    let result = result
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    
    // Restore paragraph breaks by looking for sentence endings
    let result = result
        .replace(". ", ".\n")
        .replace("! ", "!\n")
        .replace("? ", "?\n");
    
    // Clean up any double newlines
    let result = result
        .replace("\n\n", "\n")
        .trim()
        .to_string();
    
    result
}