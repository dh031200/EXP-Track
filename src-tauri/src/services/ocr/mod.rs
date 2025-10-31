pub mod parser;
pub mod http_ocr;
pub mod template_matcher;
pub mod inventory_template_matcher;

// Re-export main types
pub use http_ocr::HttpOcrClient;
pub use inventory_template_matcher::InventoryTemplateMatcher;
