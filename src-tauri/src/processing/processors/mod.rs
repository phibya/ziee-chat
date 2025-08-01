pub mod text;
pub mod image;
pub mod pdf;
pub mod office;
pub mod spreadsheet;

pub use text::{TextProcessor, TextImageGenerator};
pub use image::{ImageProcessor, ImageGenerator};
pub use pdf::{PdfProcessor, PdfImageGenerator};
pub use office::{OfficeProcessor, OfficeImageGenerator};
pub use spreadsheet::{SpreadsheetProcessor, SpreadsheetImageGenerator};