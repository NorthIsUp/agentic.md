pub mod copilot;
pub mod cursor;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}
