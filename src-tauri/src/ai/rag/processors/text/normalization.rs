// Text normalization functionality for entities and relationships

use crate::ai::rag::{RAGErrorCode, RAGResult, RAGIndexingErrorCode};
use regex::Regex;
use unicode_normalization::{char::canonical_combining_class, UnicodeNormalization};

/// Unicode normalization form options
#[derive(Debug, Clone, Copy)]
pub enum UnicodeNormalizationForm {
    /// Canonical Decomposition, followed by Canonical Composition (recommended for most use cases)
    NFC,
    /// Canonical Decomposition
    NFD,
    /// Compatibility Decomposition, followed by Canonical Composition (more aggressive)
    NFKC,
    /// Compatibility Decomposition
    NFKD,
}

impl Default for UnicodeNormalizationForm {
    fn default() -> Self {
        UnicodeNormalizationForm::NFC
    }
}

/// Unified text normalization and cleaning utilities
pub struct TextNormalizer;

impl TextNormalizer {
    pub fn new() -> Self {
        Self
    }

    // ==== Basic Text Cleaning Methods ====

    /// Remove excessive whitespace while preserving structure
    pub fn clean_whitespace(text: &str) -> String {
        // Replace multiple spaces with single space
        let mut cleaned = text.trim().to_string();

        // Normalize line breaks
        cleaned = cleaned.replace("\r\n", "\n").replace("\r", "\n");

        // Remove excessive blank lines (more than 2 consecutive)
        while cleaned.contains("\n\n\n") {
            cleaned = cleaned.replace("\n\n\n", "\n\n");
        }

        cleaned
    }

    /// Remove control characters but preserve printable characters
    pub fn remove_control_chars(text: &str) -> String {
        text.chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect()
    }

    // ==== Unicode Normalization Methods ====

    /// Normalize unicode characters using proper Unicode normalization
    pub fn normalize_unicode(text: &str) -> String {
        // Use NFC (Canonical Decomposition, followed by Canonical Composition)
        // This is the recommended normalization for most text processing tasks
        text.nfc().collect()
    }

    /// Normalize unicode characters using NFD (Canonical Decomposition)
    pub fn normalize_unicode_nfd(text: &str) -> String {
        text.nfd().collect()
    }

    /// Normalize unicode characters using NFKC (Compatibility Decomposition, followed by Canonical Composition)
    /// This is more aggressive and converts compatibility characters to their canonical forms
    pub fn normalize_unicode_nfkc(text: &str) -> String {
        text.nfkc().collect()
    }

    /// Normalize unicode characters using NFKD (Compatibility Decomposition)
    pub fn normalize_unicode_nfkd(text: &str) -> String {
        text.nfkd().collect()
    }

    /// Advanced Unicode cleaning that removes zero-width characters and normalizes
    pub fn clean_and_normalize_unicode(text: &str) -> String {
        let mut result = String::new();

        for ch in text.nfc() {
            match ch {
                // Remove zero-width characters that can cause issues
                '\u{200B}' | // Zero Width Space
                '\u{200C}' | // Zero Width Non-Joiner
                '\u{200D}' | // Zero Width Joiner
                '\u{FEFF}' | // Zero Width No-Break Space (BOM)
                '\u{061C}' | // Arabic Letter Mark
                '\u{2060}' | // Word Joiner
                '\u{2061}' | // Function Application
                '\u{2062}' | // Invisible Times
                '\u{2063}' | // Invisible Separator
                '\u{2064}' | // Invisible Plus
                '\u{2066}' | // Left-to-Right Isolate
                '\u{2067}' | // Right-to-Left Isolate
                '\u{2068}' | // First Strong Isolate
                '\u{2069}'   // Pop Directional Isolate
                => continue, // Skip these characters
                
                // Keep everything else
                _ => result.push(ch),
            }
        }

        result
    }

    /// Check if text contains potentially problematic Unicode characters
    pub fn has_unicode_anomalies(text: &str) -> bool {
        for ch in text.chars() {
            match ch {
                // Check for various problematic Unicode ranges
                '\u{200B}'..='\u{200F}' | // Zero-width and directional marks
                '\u{202A}'..='\u{202E}' | // Directional formatting
                '\u{2060}'..='\u{206F}' | // Various invisible characters
                '\u{FFF0}'..='\u{FFFF}'   // Specials block
                // Note: Surrogate pairs (U+D800..U+DFFF) are invalid in UTF-8 and handled separately
                => return true,
                _ if canonical_combining_class(ch) != 0 && ch.is_control() => return true,
                _ => continue,
            }
        }
        false
    }

    // ==== Entity and Relationship Processing Methods =====

    /// Normalize extracted entity/relation names and descriptions (matching LightRAG's implementation)
    /// This method handles entity/relation name normalization from simple_vector.rs
    pub async fn normalize_extracted_info(&self, name: &str, is_entity: bool) -> RAGResult<String> {
        let mut name = name.to_string();

        // Replace Chinese parentheses with English parentheses
        name = name.replace("（", "(").replace("）", ")");

        // Replace Chinese dash with English dash
        name = name.replace("—", "-").replace("－", "-");

        // Use regex to remove spaces between Chinese characters
        let chinese_space_regex = Regex::new(r"(?<=[\u4e00-\u9fa5])\s+(?=[\u4e00-\u9fa5])")
            .map_err(|e| {
                tracing::error!("Failed to create Chinese space regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        name = chinese_space_regex.replace_all(&name, "").to_string();

        // Remove spaces between Chinese and English/numbers/symbols
        let chinese_en_regex =
            Regex::new(r"(?<=[\u4e00-\u9fa5])\s+(?=[a-zA-Z0-9\(\)\[\]@#$%!&\*\-=+_])")
                .map_err(|e| {
                    tracing::error!("Failed to create Chinese-English regex: {}", e);
                    RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
                })?;
        name = chinese_en_regex.replace_all(&name, "").to_string();

        let en_chinese_regex =
            Regex::new(r"(?<=[a-zA-Z0-9\(\)\[\]@#$%!&\*\-=+_])\s+(?=[\u4e00-\u9fa5])")
                .map_err(|e| {
                    tracing::error!("Failed to create English-Chinese regex: {}", e);
                    RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
                })?;
        name = en_chinese_regex.replace_all(&name, "").to_string();

        // Remove English quotation marks from the beginning and end
        if name.len() >= 2 && name.starts_with('"') && name.ends_with('"') {
            name = name[1..name.len() - 1].to_string();
        }
        if name.len() >= 2 && name.starts_with('\'') && name.ends_with('\'') {
            name = name[1..name.len() - 1].to_string();
        }

        if is_entity {
            // Remove Chinese quotes
            name = name
                .replace("\"", "")
                .replace("\"", "")
                .replace("'", "")
                .replace("'", "");

            // Remove English quotes in and around chinese
            let quote_chinese_regex = Regex::new(r#"['"]+(?=[\u4e00-\u9fa5])"#)
                .map_err(|e| {
                    tracing::error!("Failed to create quote-Chinese regex: {}", e);
                    RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
                })?;
            name = quote_chinese_regex.replace_all(&name, "").to_string();

            let chinese_quote_regex = Regex::new(r#"(?<=[\u4e00-\u9fa5])['"]+"#)
                .map_err(|e| {
                    tracing::error!("Failed to create Chinese-quote regex: {}", e);
                    RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
                })?;
            name = chinese_quote_regex.replace_all(&name, "").to_string();
        }

        Ok(name)
    }

    /// Normalize Chinese text by handling various formatting issues
    pub async fn normalize_chinese_text(&self, text: &str) -> RAGResult<String> {
        let mut normalized = text.to_string();

        // Replace full-width characters with half-width equivalents
        normalized = normalized
            .replace("，", ",")
            .replace("。", ".")
            .replace("！", "!")
            .replace("？", "?")
            .replace("：", ":")
            .replace("；", ";")
            .replace("（", "(")
            .replace("）", ")")
            .replace("【", "[")
            .replace("】", "]")
            .replace("「", "\"")
            .replace("」", "\"")
            .replace("『", "'")
            .replace("』", "'");

        // Handle Chinese spacing issues
        let chinese_space_regex = Regex::new(r"([\u4e00-\u9fa5])\s+([\u4e00-\u9fa5])")
            .map_err(|e| {
                tracing::error!("Failed to create Chinese space normalization regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        normalized = chinese_space_regex
            .replace_all(&normalized, "$1$2")
            .to_string();

        Ok(normalized)
    }

    /// Normalize text for entity extraction by removing problematic characters
    pub async fn normalize_for_entity_extraction(&self, text: &str) -> RAGResult<String> {
        let mut normalized = text.to_string();

        // Remove markdown-style formatting
        normalized = self.remove_markdown_formatting(&normalized).await?;

        // Remove extra whitespace
        let whitespace_regex = Regex::new(r"\s+")
            .map_err(|e| {
                tracing::error!("Failed to create whitespace regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        normalized = whitespace_regex.replace_all(&normalized, " ").to_string();

        // Normalize quotes and punctuation
        normalized = normalized
            .replace("\u{201C}", "\"")
            .replace("\u{201D}", "\"")
            .replace("'", "'")
            .replace("'", "'")
            .replace("…", "...");

        Ok(normalized.trim().to_string())
    }

    /// Remove common markdown formatting from text
    pub async fn remove_markdown_formatting(&self, text: &str) -> RAGResult<String> {
        let mut cleaned = text.to_string();

        // Remove headers
        let header_regex = Regex::new(r"^#+\s*")
            .map_err(|e| {
                tracing::error!("Failed to create header regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        cleaned = header_regex.replace_all(&cleaned, "").to_string();

        // Remove emphasis (bold/italic)
        let emphasis_regex = Regex::new(r"\*{1,2}([^*]+)\*{1,2}")
            .map_err(|e| {
                tracing::error!("Failed to create emphasis regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        cleaned = emphasis_regex.replace_all(&cleaned, "$1").to_string();

        let underscore_emphasis_regex = Regex::new(r"_{1,2}([^_]+)_{1,2}")
            .map_err(|e| {
                tracing::error!("Failed to create underscore emphasis regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        cleaned = underscore_emphasis_regex
            .replace_all(&cleaned, "$1")
            .to_string();

        // Remove code blocks and inline code
        let code_block_regex = Regex::new(r"```[\s\S]*?```")
            .map_err(|e| {
                tracing::error!("Failed to create code block regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        cleaned = code_block_regex.replace_all(&cleaned, "").to_string();

        let inline_code_regex = Regex::new(r"`([^`]+)`")
            .map_err(|e| {
                tracing::error!("Failed to create inline code regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        cleaned = inline_code_regex.replace_all(&cleaned, "$1").to_string();

        // Remove links but keep the text
        let link_regex = Regex::new(r"\[([^\]]*)\]\([^)]*\)")
            .map_err(|e| {
                tracing::error!("Failed to create link regex: {}", e);
                RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError)
            })?;
        cleaned = link_regex.replace_all(&cleaned, "$1").to_string();

        Ok(cleaned)
    }

    /// Normalize text case while preserving entity names
    pub async fn normalize_case_preserving_entities(&self, text: &str) -> RAGResult<String> {
        // First apply Unicode normalization
        let normalized = Self::normalize_unicode(text);

        // Remove problematic Unicode characters
        let cleaned = Self::clean_and_normalize_unicode(&normalized);

        Ok(cleaned.trim().to_string())
    }

    /// Comprehensive Unicode normalization for text processing
    pub async fn normalize_unicode_comprehensive(&self, text: &str) -> RAGResult<String> {
        // Check for Unicode anomalies first
        if Self::has_unicode_anomalies(text) {
            tracing::warn!("Unicode anomalies detected in text");
        }

        // Apply comprehensive Unicode cleaning and normalization
        let normalized = Self::clean_and_normalize_unicode(text);

        Ok(normalized)
    }

    /// Normalize text with specific Unicode normalization form
    pub async fn normalize_unicode_with_form(
        &self,
        text: &str,
        form: UnicodeNormalizationForm,
    ) -> RAGResult<String> {
        let normalized = match form {
            UnicodeNormalizationForm::NFC => Self::normalize_unicode(text),
            UnicodeNormalizationForm::NFD => Self::normalize_unicode_nfd(text),
            UnicodeNormalizationForm::NFKC => Self::normalize_unicode_nfkc(text),
            UnicodeNormalizationForm::NFKD => Self::normalize_unicode_nfkd(text),
        };

        Ok(normalized)
    }
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self::new()
    }
}
