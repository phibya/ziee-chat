use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ContentProcessor;

pub struct VideoProcessor;

impl VideoProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn get_video_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(file_path)
            .output()?;

        if !output.status.success() {
            return Ok(serde_json::json!({
                "type": "video",
                "error": "Could not extract video metadata"
            }));
        }

        let metadata_str = String::from_utf8_lossy(&output.stdout);
        let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

        // Extract useful information
        let mut result = serde_json::json!({
            "type": "video"
        });

        if let Some(format) = metadata.get("format") {
            if let Some(duration) = format.get("duration") {
                if let Some(duration_str) = duration.as_str() {
                    if let Ok(duration_f64) = duration_str.parse::<f64>() {
                        result["duration_seconds"] = serde_json::Value::Number(serde_json::Number::from_f64(duration_f64).unwrap_or(serde_json::Number::from(0)));
                    }
                }
            }
            if let Some(size) = format.get("size") {
                result["file_size"] = size.clone();
            }
            if let Some(bit_rate) = format.get("bit_rate") {
                result["bit_rate"] = bit_rate.clone();
            }
        }

        if let Some(streams) = metadata.get("streams") {
            if let Some(streams_array) = streams.as_array() {
                for stream in streams_array {
                    if let Some(codec_type) = stream.get("codec_type") {
                        if codec_type == "video" {
                            if let Some(width) = stream.get("width") {
                                result["width"] = width.clone();
                            }
                            if let Some(height) = stream.get("height") {
                                result["height"] = height.clone();
                            }
                            if let Some(codec_name) = stream.get("codec_name") {
                                result["video_codec"] = codec_name.clone();
                            }
                            break;
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl ContentProcessor for VideoProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                "video/mp4" |
                "video/quicktime" |
                "video/x-msvideo" |
                "video/webm" |
                "video/ogg" |
                "video/x-flv" |
                "video/3gpp" |
                "video/x-ms-wmv"
            )
        } else {
            false
        }
    }

    async fn extract_text(&self, _file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Videos don't have extractable text content (unless we implement subtitle extraction)
        Ok(None)
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        match self.get_video_metadata(file_path).await {
            Ok(metadata) => Ok(metadata),
            Err(_) => {
                let file_metadata = std::fs::metadata(file_path)?;
                Ok(serde_json::json!({
                    "type": "video",
                    "file_size": file_metadata.len(),
                    "error": "Could not extract video metadata"
                }))
            }
        }
    }

    async fn to_base64(&self, _file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Videos are too large for base64 encoding
        Ok(None)
    }
}