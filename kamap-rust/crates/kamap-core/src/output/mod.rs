pub mod text;
pub mod json;

pub use text::*;
pub use json::*;

/// 输出模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputMode {
    Text,
    Json,
}

impl OutputMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputMode::Json,
            _ => OutputMode::Text,
        }
    }
}
