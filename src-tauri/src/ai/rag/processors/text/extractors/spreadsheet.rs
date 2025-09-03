// Spreadsheet extractor with enhanced CSV/Excel/ODS support

use super::base::{ExtractionError, TextExtractor};
use crate::ai::rag::{RAGErrorCode, RAGResult, RAGIndexingErrorCode};
use async_trait::async_trait;

/// Enhanced spreadsheet extractor supporting CSV, Excel, and OpenDocument formats
pub struct SpreadsheetExtractor {
    file_path: String,
}

impl SpreadsheetExtractor {
    /// Extract text from CSV content and convert to Markdown table
    async fn extract_csv_to_markdown(&self) -> RAGResult<String> {
        let content = std::fs::read(&self.file_path).map_err(|e| {
            tracing::error!("Failed to read CSV file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;
        let csv_content = super::decode_text_content(&content)?;

        // Parse CSV and convert to Markdown table format
        let lines: Vec<&str> = csv_content.lines().collect();
        if lines.is_empty() {
            return Ok("*[Empty CSV file]*".to_string());
        }

        let mut markdown = String::new();
        let mut headers: Vec<String> = Vec::new();

        // Parse header row
        if let Some(header_line) = lines.first() {
            headers = self.parse_csv_row(header_line);

            // Create Markdown table header
            if !headers.is_empty() {
                markdown.push_str("# CSV Data\n\n");
                markdown.push_str(&format!("| {} |\n", headers.join(" | ")));
                markdown.push_str(&format!("| {} |\n", vec!["---"; headers.len()].join(" | ")));
            }
        }

        // Process data rows as table rows
        let mut row_count = 0;
        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let values = self.parse_csv_row(line);
            if !values.is_empty() {
                // Pad values to match header count
                let mut padded_values = values;
                while padded_values.len() < headers.len() {
                    padded_values.push(String::new());
                }
                padded_values.truncate(headers.len());

                // Escape Markdown characters in cell values
                let escaped_values: Vec<String> = padded_values
                    .iter()
                    .map(|v| self.escape_markdown_chars(v))
                    .collect();

                markdown.push_str(&format!("| {} |\n", escaped_values.join(" | ")));
                row_count += 1;
            }
        }

        if row_count == 0 {
            markdown.push_str("\n*[No data rows found]*\n");
        }

        Ok(markdown)
    }

    /// Parse a CSV row handling quoted values and escapes
    fn parse_csv_row(&self, line: &str) -> Vec<String> {
        let mut values = Vec::new();
        let mut current_value = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        // Escaped quote
                        current_value.push('"');
                        chars.next();
                    } else {
                        // Toggle quote state
                        in_quotes = !in_quotes;
                    }
                }
                ',' if !in_quotes => {
                    // End of field
                    values.push(current_value.trim().to_string());
                    current_value.clear();
                }
                _ => {
                    current_value.push(ch);
                }
            }
        }

        // Add the last value
        values.push(current_value.trim().to_string());
        values
    }

    /// Escape Markdown special characters in spreadsheet cell values
    fn escape_markdown_chars(&self, text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('*', "\\*")
            .replace('_', "\\_")
            .replace('`', "\\`")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('#', "\\#")
            .replace('+', "\\+")
            .replace('-', "\\-")
            .replace('.', "\\.")
            .replace('!', "\\!")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('|', "\\|")
            .replace('>', "\\>")
    }

    /// Extract text from Excel files using calamine and convert to Markdown
    async fn extract_excel_to_markdown(&self, is_xlsx: bool) -> RAGResult<String> {
        let content = std::fs::read(&self.file_path).map_err(|e| {
            tracing::error!("Failed to read Excel file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;
        let content_vec = content.to_vec();

        let result = tokio::task::spawn_blocking(move || {
            use calamine::{open_workbook_from_rs, Xls, Xlsx};
            use std::io::Cursor;

            let cursor = Cursor::new(content_vec);

            if is_xlsx {
                let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
                    .map_err(|e| format!("Failed to open XLSX: {}", e))?;
                Self::extract_from_xlsx_workbook_to_markdown(&mut workbook)
            } else {
                let mut workbook: Xls<_> = open_workbook_from_rs(cursor)
                    .map_err(|e| format!("Failed to open XLS: {}", e))?;
                Self::extract_from_xls_workbook_to_markdown(&mut workbook)
            }
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute Excel extraction task for file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?
        .map_err(|e| {
            tracing::error!("Excel extraction failed for file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;

        Ok(result)
    }

    /// Extract text from Excel workbook (XLSX) as Markdown
    fn extract_from_xlsx_workbook_to_markdown(
        workbook: &mut calamine::Xlsx<std::io::Cursor<Vec<u8>>>,
    ) -> Result<String, String> {
        use calamine::Reader;

        let sheet_names: Vec<String> = workbook
            .sheet_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut all_sheets_markdown = Vec::new();

        all_sheets_markdown.push("# Excel Spreadsheet Data\n".to_string());

        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let sheet_markdown = Self::extract_from_range_to_markdown(&range, &sheet_name);
                if !sheet_markdown.trim().is_empty() {
                    all_sheets_markdown.push(sheet_markdown);
                }
            }
        }

        Ok(all_sheets_markdown.join("\n\n"))
    }

    /// Extract text from Excel workbook (XLS) as Markdown
    fn extract_from_xls_workbook_to_markdown(
        workbook: &mut calamine::Xls<std::io::Cursor<Vec<u8>>>,
    ) -> Result<String, String> {
        use calamine::Reader;

        let sheet_names: Vec<String> = workbook
            .sheet_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut all_sheets_markdown = Vec::new();

        all_sheets_markdown.push("# Excel Spreadsheet Data\n".to_string());

        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let sheet_markdown = Self::extract_from_range_to_markdown(&range, &sheet_name);
                if !sheet_markdown.trim().is_empty() {
                    all_sheets_markdown.push(sheet_markdown);
                }
            }
        }

        Ok(all_sheets_markdown.join("\n\n"))
    }

    /// Extract text from OpenDocument spreadsheet as Markdown
    fn extract_from_ods_workbook_to_markdown(
        workbook: &mut calamine::Ods<std::io::Cursor<Vec<u8>>>,
    ) -> Result<String, String> {
        use calamine::Reader;

        let sheet_names: Vec<String> = workbook
            .sheet_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut all_sheets_markdown = Vec::new();

        all_sheets_markdown.push("# OpenDocument Spreadsheet Data\n".to_string());

        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let sheet_markdown = Self::extract_from_range_to_markdown(&range, &sheet_name);
                if !sheet_markdown.trim().is_empty() {
                    all_sheets_markdown.push(sheet_markdown);
                }
            }
        }

        Ok(all_sheets_markdown.join("\n\n"))
    }

    /// Extract text from a calamine range as Markdown table
    fn extract_from_range_to_markdown(
        range: &calamine::Range<calamine::Data>,
        sheet_name: &str,
    ) -> String {
        let mut markdown = String::new();

        if !range.is_empty() {
            markdown.push_str(&format!("## {}\n\n", sheet_name));

            // Find the actual data bounds
            let (height, width) = range.get_size();

            if height == 0 || width == 0 {
                markdown.push_str("*[Empty sheet]*\n");
                return markdown;
            }

            // Create table header and separator
            let mut has_header = false;

            // Extract all data first
            let mut table_data: Vec<Vec<String>> = Vec::new();
            for row in 0..height {
                let mut row_values = Vec::new();
                let mut has_data = false;

                for col in 0..width {
                    let cell_text = if let Some(cell) = range.get((row, col)) {
                        let text = Self::format_cell_value(cell);
                        if !text.is_empty() {
                            has_data = true;
                        }
                        Self::escape_markdown_in_cell(&text)
                    } else {
                        String::new()
                    };
                    row_values.push(cell_text);
                }

                if has_data {
                    table_data.push(row_values);
                }
            }

            if table_data.is_empty() {
                markdown.push_str("*[No data in sheet]*\n");
                return markdown;
            }

            // Create Markdown table
            for (row_idx, row_data) in table_data.iter().enumerate() {
                markdown.push_str(&format!("| {} |\n", row_data.join(" | ")));

                // Add table separator after first row (header)
                if row_idx == 0 && table_data.len() > 1 {
                    let separator_cells: Vec<&str> = vec!["---"; row_data.len()];
                    markdown.push_str(&format!("| {} |\n", separator_cells.join(" | ")));
                    has_header = true;
                }
            }

            if !has_header && !table_data.is_empty() {
                // No header was added, so this is a single row table
                // Add a separator anyway for proper Markdown table format
                let separator_cells: Vec<&str> = vec!["---"; table_data[0].len()];
                markdown.push_str(&format!("| {} |\n", separator_cells.join(" | ")));
            }
        } else {
            markdown.push_str(&format!("## {}\n\n*[Empty sheet]*\n", sheet_name));
        }

        markdown
    }

    /// Escape Markdown characters specifically for table cells
    fn escape_markdown_in_cell(text: &str) -> String {
        // In table cells, we mainly need to escape pipe characters and newlines
        text.replace('|', "\\|")
            .replace('\n', " ")
            .replace('\r', " ")
            .replace('\t', " ")
    }

    /// Format cell value for text output
    fn format_cell_value(cell: &calamine::Data) -> String {
        match cell {
            calamine::Data::Empty => String::new(),
            calamine::Data::String(s) => s.clone(),
            calamine::Data::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Bool(b) => b.to_string(),
            calamine::Data::DateTime(dt) => format!("{:.0}", dt),
            calamine::Data::Error(e) => format!("[Error: {:?}]", e),
            calamine::Data::DateTimeIso(dt) => dt.clone(),
            calamine::Data::DurationIso(d) => d.clone(),
        }
    }

    /// Extract text from OpenDocument Spreadsheet and convert to Markdown
    async fn extract_ods_to_markdown(&self) -> RAGResult<String> {
        let content = std::fs::read(&self.file_path).map_err(|e| {
            tracing::error!("Failed to read ODS file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;
        let content_vec = content.to_vec();

        let result = tokio::task::spawn_blocking(move || {
            use calamine::{open_workbook_from_rs, Ods};
            use std::io::Cursor;

            let cursor = Cursor::new(content_vec);
            let mut workbook: Ods<_> =
                open_workbook_from_rs(cursor).map_err(|e| format!("Failed to open ODS: {}", e))?;

            Self::extract_from_ods_workbook_to_markdown(&mut workbook)
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute ODS extraction task for file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?
        .map_err(|e| {
            tracing::error!("ODS extraction failed for file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;

        Ok(result)
    }

    /// Detect spreadsheet format from content
    fn detect_format_from_content(&self, content: &[u8]) -> Option<&'static str> {
        // Check for Excel signatures
        if content.starts_with(b"PK\x03\x04") {
            // ZIP-based format (XLSX or ODS)
            let content_str = String::from_utf8_lossy(&content[..1000]);
            if content_str.contains("xl/") || content_str.contains("worksheets/") {
                return Some("xlsx");
            } else if content_str.contains("content.xml") && content_str.contains("spreadsheet") {
                return Some("ods");
            }
        }

        // Check for old Excel format
        if content.starts_with(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1") {
            return Some("xls");
        }

        // Check for CSV (basic heuristic)
        if let Ok(text) = String::from_utf8(content[..content.len().min(1000)].to_vec()) {
            if text.contains(',') || text.contains('\t') {
                let lines: Vec<&str> = text.lines().take(5).collect();
                if lines.len() > 1 {
                    let comma_counts: Vec<usize> =
                        lines.iter().map(|line| line.matches(',').count()).collect();

                    // If multiple lines have similar comma counts, it's likely CSV
                    if comma_counts.iter().filter(|&&count| count > 0).count() > 1 {
                        return Some("csv");
                    }
                }
            }
        }

        None
    }
}

#[async_trait]
impl TextExtractor for SpreadsheetExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Always convert to Markdown first (unified approach)
        let content = std::fs::read(&self.file_path).map_err(|e| {
            tracing::error!("Failed to read spreadsheet file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;
        let format = self.detect_format_from_content(&content);

        match format {
            Some("csv") => self.extract_csv_to_markdown().await,
            Some("xlsx") => self.extract_excel_to_markdown(true).await,
            Some("xls") => self.extract_excel_to_markdown(false).await,
            Some("ods") => self.extract_ods_to_markdown().await,
            _ => Err(ExtractionError::InvalidFormat(
                "Unknown or unsupported spreadsheet format".to_string(),
            )
            .into()),
        }
    }
}
