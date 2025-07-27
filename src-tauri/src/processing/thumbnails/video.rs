use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ThumbnailGenerator;

pub struct VideoThumbnailGenerator;

impl VideoThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

    async fn generate_video_thumbnail_with_ffmpeg(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let thumbnail_path = output_dir.join("page_1.jpg");

        // Extract frame at 1 second (or 10% of duration, whichever is smaller)
        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(file_path)
            .arg("-ss")
            .arg("00:00:01") // Seek to 1 second
            .arg("-vframes")
            .arg("1") // Extract 1 frame
            .arg("-vf")
            .arg("scale=300:300:force_original_aspect_ratio=decrease") // Scale to fit 300x300
            .arg("-y") // Overwrite output file
            .arg(&thumbnail_path)
            .output()?;

        if !output.status.success() {
            // Try extracting from the first frame if seeking fails
            let output = Command::new("ffmpeg")
                .arg("-i")
                .arg(file_path)
                .arg("-vframes")
                .arg("1")
                .arg("-vf")
                .arg("scale=300:300:force_original_aspect_ratio=decrease")
                .arg("-y")
                .arg(&thumbnail_path)
                .output()?;

            if !output.status.success() {
                return Err(format!("FFmpeg thumbnail generation failed: {}", String::from_utf8_lossy(&output.stderr)).into());
            }
        }

        if thumbnail_path.exists() {
            Ok(1)
        } else {
            Err("Thumbnail file was not created".into())
        }
    }
}

#[async_trait]
impl ThumbnailGenerator for VideoThumbnailGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
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

    async fn generate_thumbnails(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        self.generate_video_thumbnail_with_ffmpeg(file_path, output_dir).await
    }
}