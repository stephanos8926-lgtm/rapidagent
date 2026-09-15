//! Core module for rw-syspro-compiler
//!
//! Provides lexer, parser, rag parser, and optimizer for prompt compiler source files.

pub mod lexer;
pub mod parser;
pub mod rag_parser;
pub mod optimizer;

pub use lexer::{Lexer, Token};
pub use parser::{Parser, parse_tokens, compile_source};
pub use rag_parser::{parse_rag, RagArtifact, RagFrontmatter, RagSection, RagGraph, RagNode, RagEdge};
pub use optimizer::optimize;