// Office document extractor with Pandoc integration

use super::base::TextExtractor;
use crate::ai::rag::{RAGErrorCode, RAGIndexingErrorCode, RAGResult};
use crate::utils::pandoc::PandocUtils;
use async_trait::async_trait;
use std::process::Command;

/// Office document extractor supporting Word, RTF, OpenDocument formats and other documents (excluding PPTX)
pub struct OfficeExtractor {
    file_path: String,
}

impl OfficeExtractor {
    /// Check if Pandoc is available and get its path
    fn get_pandoc_path() -> Option<std::path::PathBuf> {
        PandocUtils::get_pandoc_path()
    }

    /// Extract text from office documents using Pandoc
    async fn extract_with_pandoc(&self, format_hint: Option<&str>) -> RAGResult<String> {
        let pandoc_path = Self::get_pandoc_path().ok_or_else(|| {
            tracing::error!(
                "Pandoc not found. Office document extraction requires Pandoc for file: {}",
                self.file_path
            );
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;

        // Determine input format arguments based on format hint or MIME type
        let format_args = match format_hint {
            Some("docx") => vec!["--from", "docx"],
            Some("doc") => vec!["--from", "doc"],
            Some("rtf") => vec!["--from", "rtf"],
            Some("odt") => vec!["--from", "odt"],
            Some("odp") => vec!["--from", "odp"],
            Some("epub") => vec!["--from", "epub"],
            Some("fb2") => vec!["--from", "fb2"],
            Some("latex") => vec!["--from", "latex"],
            Some("ipynb") => vec!["--from", "ipynb"],
            Some("docbook") => vec!["--from", "docbook"],
            Some("jats") => vec!["--from", "jats"],
            Some("mediawiki") => vec!["--from", "mediawiki"],
            _ => vec![], // Let Pandoc auto-detect
        };

        // Use Pandoc to convert document to markdown with enhanced formatting
        let mut cmd = Command::new(&pandoc_path);
        cmd.arg(&self.file_path);

        // Add format arguments if specified
        for arg in format_args {
            cmd.arg(arg);
        }

        cmd.arg("--to")
            .arg("markdown")
            .arg("--wrap=none") // Don't wrap lines
            .arg("--standalone") // Include metadata
            .arg("--extract-media=."); // Extract embedded media info

        let output = cmd.output().map_err(|e| {
            tracing::error!(
                "Failed to run Pandoc for Office document {}: {}",
                self.file_path,
                e
            );
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            tracing::error!(
                "Pandoc conversion failed for Office document {}: {}",
                self.file_path,
                error_msg
            );
            return Err(RAGErrorCode::Indexing(
                RAGIndexingErrorCode::TextExtractionFailed,
            ));
        }

        let markdown_content = String::from_utf8(output.stdout).map_err(|e| {
            tracing::error!(
                "Invalid UTF-8 output from Pandoc for Office document {}: {}",
                self.file_path,
                e
            );
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;

        Ok(markdown_content)
    }

    /// Detect document format from content
    fn detect_format_from_content(&self, content: &[u8]) -> Option<&'static str> {
        // Check ZIP-based formats (DOCX, EPUB, ODT, etc.)
        if content.starts_with(b"PK\x03\x04") && content.len() > 1000 {
            let content_str = String::from_utf8_lossy(&content[..2000]);

            // Office Open XML formats (documents only)
            if content_str.contains("word/") || content_str.contains("document.xml") {
                return Some("docx");
            }
            // Note: PPTX detection removed - handled by PptxExtractor
            // Note: XLSX detection removed - handled by SpreadsheetExtractor

            // OpenDocument formats (documents and presentations only)
            if content_str.contains("mimetype") {
                if content_str.contains("opendocument.text") {
                    return Some("odt");
                }
                if content_str.contains("opendocument.presentation") {
                    return Some("odp");
                }
                // Note: ODS detection removed - handled by SpreadsheetExtractor
            }

            // EPUB format
            if content_str.contains("epub") || content_str.contains("META-INF/container.xml") {
                return Some("epub");
            }
        }

        // RTF format
        if content.starts_with(b"{\\rtf1") {
            return Some("rtf");
        }

        // Old Office formats (OLE compound document)
        if content.starts_with(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1") {
            // This could be DOC, XLS, or PPT - let Pandoc auto-detect
            // Most commonly this will be DOC files
            return Some("doc");
        }

        // FictionBook2 format
        if content.starts_with(b"<?xml") {
            let content_str = String::from_utf8_lossy(&content[..500]);
            if content_str.contains("FictionBook") {
                return Some("fb2");
            }
            if content_str.contains("DocBook") || content_str.contains("docbook") {
                return Some("docbook");
            }
            if content_str.contains("jats") || content_str.contains("JATS") {
                return Some("jats");
            }
        }

        // LaTeX format
        if content.starts_with(b"\\documentclass") || content.starts_with(b"\\begin{document}") {
            return Some("latex");
        }

        // Jupyter notebook (JSON format with specific structure)
        if content.starts_with(b"{") {
            let content_str = String::from_utf8_lossy(&content[..500]);
            if content_str.contains("\"cells\"") && content_str.contains("\"metadata\"") {
                return Some("ipynb");
            }
        }

        // MediaWiki format (basic detection)
        let content_str = String::from_utf8_lossy(&content[..200]);
        if content_str.contains("{{!") || content_str.contains("[[Category:") {
            return Some("mediawiki");
        }

        // Note: CSV/TSV detection removed - handled by SpreadsheetExtractor

        None
    }
}

#[async_trait]
impl TextExtractor for OfficeExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Always convert to Markdown first (unified approach)
        let content = std::fs::read(&self.file_path).map_err(|e| {
            tracing::error!("Failed to read Office document {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;
        let format_hint = self.detect_format_from_content(&content);
        self.extract_with_pandoc(format_hint).await
    }
}
