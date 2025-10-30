pub mod parser;
pub mod http_ocr;
pub mod template_matcher;

// Re-export main types
pub use http_ocr::HttpOcrClient;
pub use template_matcher::TemplateMatcher;
