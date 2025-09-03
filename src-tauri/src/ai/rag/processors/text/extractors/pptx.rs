// PowerPoint PPTX extractor using pptx-to-md crate

use super::base::TextExtractor;
use crate::ai::rag::{RAGErrorCode, RAGResult, RAGIndexingErrorCode};
use async_trait::async_trait;
use pptx_to_md::{ImageHandlingMode, ParserConfig, PptxContainer};
use std::path::Path;

/// PowerPoint PPTX extractor using pptx-to-md crate
pub struct PptxExtractor {
    file_path: String,
}

impl PptxExtractor {
    /// Extract content from PPTX file using pptx-to-md crate
    async fn extract_pptx_content(&self) -> RAGResult<String> {
        // Verify file exists and is readable
        let path = Path::new(&self.file_path);
        if !path.exists() {
            tracing::error!("PPTX file does not exist: {}", self.file_path);
            return Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed));
        }

        // Use the pptx-to-md crate with correct API based on examples
        let config = ParserConfig::builder()
            .extract_images(true)
            .compress_images(true)
            .quality(80)
            .image_handling_mode(ImageHandlingMode::InMarkdown)
            .include_slide_comment(true)
            .build();

        let path = Path::new(&self.file_path);

        // Open PPTX container and parse slides
        match PptxContainer::open(path, config) {
            Ok(mut container) => {
                // Parse all slides
                match container.parse_all() {
                    Ok(slides) => {
                        let mut all_content = String::new();

                        // Convert each slide to markdown
                        for slide in slides {
                            if let Some(md_content) = slide.convert_to_md() {
                                all_content.push_str(&md_content);
                                all_content.push_str("\n\n---\n\n");
                            }
                        }

                        if all_content.trim().is_empty() {
                            tracing::error!("No extractable content found in PPTX file: {}", self.file_path);
                            return Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed));
                        }

                        // Clean up trailing separator
                        let cleaned_content = all_content.trim_end_matches("\n\n---\n\n");
                        Ok(cleaned_content.to_string())
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse PPTX slides for file {}: {}", self.file_path, e);
                        Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed))
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to open PPTX container for file {}: {}", self.file_path, e);
                Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed))
            }
        }
    }

    /// Validate that this is a PPTX file
    fn validate_pptx_file(&self) -> RAGResult<()> {
        let path = Path::new(&self.file_path);

        // Check file extension
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.to_lowercase() == "pptx" => Ok(()),
            _ => {
                tracing::error!("Invalid PPTX file extension for file: {}", self.file_path);
                Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed))
            }
        }
    }
}

#[async_trait]
impl TextExtractor for PptxExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Validate file format
        self.validate_pptx_file()?;

        // Extract markdown content using pptx-to-md
        let markdown_content = self.extract_pptx_content().await?;

        // Clean and format the output
        let cleaned_content = self.clean_pptx_markdown(&markdown_content);

        Ok(cleaned_content)
    }
}

impl PptxExtractor {
    /// Clean and format markdown extracted from PPTX
    fn clean_pptx_markdown(&self, content: &str) -> String {
        // Remove excessive whitespace and normalize line endings
        let mut cleaned = content.trim().to_string();

        // Normalize line endings to Unix style
        cleaned = cleaned.replace("\r\n", "\n").replace('\r', "\n");

        // Remove multiple consecutive blank lines (more than 2)
        while cleaned.contains("\n\n\n\n") {
            cleaned = cleaned.replace("\n\n\n\n", "\n\n\n");
        }

        // Ensure proper spacing around headings
        cleaned = regex::Regex::new(r"\n(#{1,6})")
            .unwrap()
            .replace_all(&cleaned, "\n\n$1")
            .to_string();

        cleaned = regex::Regex::new(r"(#{1,6}[^\n]+)\n([^\n#])")
            .unwrap()
            .replace_all(&cleaned, "$1\n\n$2")
            .to_string();

        // Clean up any remaining excessive whitespace
        cleaned = regex::Regex::new(r"[ \t]+")
            .unwrap()
            .replace_all(&cleaned, " ")
            .to_string();

        cleaned.trim().to_string()
    }
}
