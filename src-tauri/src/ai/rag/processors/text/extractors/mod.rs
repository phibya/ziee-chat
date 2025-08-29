// Unified text extraction with Markdown-first approach

pub mod base;
pub mod html;
pub mod markdown;
pub mod office;
pub mod pdf;
pub mod plain_text;
pub mod spreadsheet;

use crate::ai::rag::RAGResult;
pub use base::TextExtractor;
use std::collections::HashMap;
use std::path::Path;

// Re-export extractors for internal use
pub use html::HtmlExtractor;
pub use markdown::MarkdownExtractor;
pub use office::OfficeExtractor;
pub use pdf::PdfExtractor;
pub use plain_text::PlainTextExtractor;
pub use spreadsheet::SpreadsheetExtractor;

/// Main entry point: Convert any file content to Markdown with metadata
pub async fn convert_to_markdown(
    file_path: &str,
) -> RAGResult<(String, HashMap<String, serde_json::Value>)> {
    // Read file content for encoding detection and metadata
    let content = std::fs::read(file_path).map_err(|e| {
        crate::ai::rag::RAGError::TextExtractionError(format!("Failed to read file: {}", e))
    })?;

    // Detect MIME type from file extension
    let mime_type = detect_mime_type_from_path(file_path);

    // For text-based formats, ensure UTF-8 encoding is handled before processing
    if is_text_based_format(&mime_type) {
        let _decoded_content = decode_bytes_to_utf8(&content)?;
        // Note: Individual extractors will handle their own file reading with proper encoding
    }

    // Convert to markdown using appropriate extractor
    let markdown = convert_file_to_markdown(file_path, &mime_type).await?;

    // Clean the markdown using TextNormalizer from normalization module
    let cleaned_markdown = super::normalization::TextNormalizer::clean_whitespace(&markdown);
    let cleaned_markdown =
        super::normalization::TextNormalizer::remove_control_chars(&cleaned_markdown);
    let cleaned_markdown =
        super::normalization::TextNormalizer::normalize_unicode(&cleaned_markdown);

    // Extract metadata from cleaned markdown
    let metadata =
        base::MarkdownMetadataExtractor::extract_metadata(&cleaned_markdown, &content, &mime_type)
            .await?;

    Ok((cleaned_markdown, metadata))
}

/// Convert file to markdown based on MIME type using appropriate extractor
async fn convert_file_to_markdown(file_path: &str, mime_type: &str) -> RAGResult<String> {
    match mime_type {
        // Plain text formats
        "text/plain" | "text/txt" | "application/x-empty" | "text/x-readme" | 
        "text/x-log" | "text/x-diff" | "text/x-patch" => {
            PlainTextExtractor::new(file_path).extract_to_markdown().await
        },
        
        // HTML formats
        "text/html" | "application/xhtml+xml" | "text/xhtml" | 
        "application/xhtml" | "text/x-server-parsed-html" => {
            HtmlExtractor::new(file_path).extract_to_markdown().await
        },
        
        // Markdown formats (preserve with light cleanup)
        "text/markdown" | "text/md" | "text/x-markdown" | "application/x-markdown" => {
            MarkdownExtractor::new(file_path).extract_to_markdown().await
        },
        
        // PDF formats
        "application/pdf" => {
            PdfExtractor::new(file_path).extract_to_markdown().await
        },
        
        // Spreadsheet formats
        "text/csv" | "application/csv" | "text/tab-separated-values" | "text/tsv" |
        "application/vnd.ms-excel" | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
        "application/vnd.oasis.opendocument.spreadsheet" => {
            SpreadsheetExtractor::new(file_path).extract_to_markdown().await
        },
        
        // Office document formats
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" | // DOCX
        "application/msword" | // DOC
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" | // PPTX
        "application/vnd.ms-powerpoint" | // PPT
        "application/vnd.oasis.opendocument.text" | // ODT
        "application/vnd.oasis.opendocument.presentation" | // ODP
        "application/rtf" | "text/rtf" | // RTF
        "application/epub+zip" | // EPUB
        "application/x-fictionbook+xml" | // FB2
        "application/x-latex" | "text/x-tex" | // LaTeX
        "application/x-ipynb+json" | // Jupyter
        "application/docbook+xml" | // DocBook
        "application/jats+xml" | // JATS
        "text/x-wiki" => { // MediaWiki
            OfficeExtractor::new(file_path).extract_to_markdown().await
        },
        
        _ => {
            Err(crate::ai::rag::RAGError::TextExtractionError(format!(
                "Unsupported file type: {}",
                mime_type
            )))
        }
    }
}

/// Decode bytes to UTF-8 with lossy conversion
fn decode_bytes_to_utf8(bytes: &[u8]) -> RAGResult<String> {
    Ok(String::from_utf8_lossy(bytes).to_string())
}

/// Check if the MIME type represents a text-based format that needs encoding handling
fn is_text_based_format(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "text/plain"
            | "text/txt"
            | "text/html"
            | "text/markdown"
            | "text/md"
            | "text/csv"
            | "text/tsv"
            | "application/csv"
            | "text/tab-separated-values"
            | "text/x-readme"
            | "text/x-log"
            | "text/x-diff"
            | "text/x-patch"
            | "application/xhtml+xml"
            | "text/xhtml"
            | "application/xhtml"
            | "text/x-server-parsed-html"
            | "text/x-markdown"
            | "application/x-markdown"
            | "application/rtf"
            | "text/rtf"
            | "application/x-latex"
            | "text/x-tex"
            | "text/x-wiki"
    )
}

/// Detect MIME type from file extension
fn detect_mime_type_from_path(file_path: &str) -> String {
    let path = Path::new(file_path);

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("txt") | Some("log") | Some("diff") | Some("patch") => "text/plain",
        Some("html") | Some("htm") => "text/html",
        Some("md") | Some("markdown") => "text/markdown",
        Some("pdf") => "application/pdf",
        Some("csv") => "text/csv",
        Some("tsv") => "text/tab-separated-values",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("xls") => "application/vnd.ms-excel",
        Some("ods") => "application/vnd.oasis.opendocument.spreadsheet",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("doc") => "application/msword",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("odt") => "application/vnd.oasis.opendocument.text",
        Some("odp") => "application/vnd.oasis.opendocument.presentation",
        Some("rtf") => "application/rtf",
        Some("epub") => "application/epub+zip",
        Some("tex") => "application/x-latex",
        Some("ipynb") => "application/x-ipynb+json",
        _ => "text/plain", // Default fallback
    }
    .to_string()
}

/// Public utility function for encoding detection (used by extractors)
pub fn decode_text_content(bytes: &[u8]) -> RAGResult<String> {
    decode_bytes_to_utf8(bytes)
}
