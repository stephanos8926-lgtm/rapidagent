//! Renderer module for rw-syspro-compiler
//!
//! Converts PromptIR to various output formats (XML, Markdown, JSON, Folder, Binary).

pub mod xml;
pub mod markdown;
pub mod json;
pub mod folder;
pub mod binary;

pub use xml::render_xml;
pub use markdown::render_markdown;
pub use json::render_json;
pub use folder::{render_folder, FolderFormat};
pub use binary::{to_binary, from_binary, CompilerIdentity};
