pub mod text;
pub mod image;
pub mod pdf;
pub mod office;
pub mod video;

pub use text::TextProcessor;
pub use image::ImageProcessor;
pub use pdf::PdfProcessor;
pub use office::OfficeProcessor;
pub use video::VideoProcessor;