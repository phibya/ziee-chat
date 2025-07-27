pub mod image;
pub mod text;
pub mod pdf;
pub mod office;
pub mod video;

pub use image::ImageThumbnailGenerator;
pub use text::TextThumbnailGenerator;
pub use pdf::PdfThumbnailGenerator;
pub use office::OfficeThumbnailGenerator;
pub use video::VideoThumbnailGenerator;

// Maximum number of thumbnails to generate for PDF files
pub const MAX_PDF_THUMBNAILS: u32 = 5;