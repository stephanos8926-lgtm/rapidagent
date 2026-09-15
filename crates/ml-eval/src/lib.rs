//! ML-based evaluation module
//!
//! Provides embedding-based similarity, quality scoring, and optional LLM review.

pub mod embeddings;
pub mod quality;
pub mod llm_review;

pub use embeddings::*;
pub use quality::*;
pub use llm_review::*;
