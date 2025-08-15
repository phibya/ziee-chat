pub mod image;
pub mod office;
pub mod pdf;
pub mod spreadsheet;
pub mod text;

pub use image::{ImageGenerator, ImageProcessor};
pub use office::{OfficeImageGenerator, OfficeProcessor};
pub use pdf::{PdfImageGenerator, PdfProcessor};
pub use spreadsheet::{SpreadsheetImageGenerator, SpreadsheetProcessor};
pub use text::{TextImageGenerator, TextProcessor};
