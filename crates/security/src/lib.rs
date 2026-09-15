//! Security module for rw-syspro-compiler
//!
//! Provides injection scanning, content hashing, and signature verification.

pub mod scanner;
pub mod hasher;
pub mod signer;

pub use scanner::{scan_for_injection, SecurityWarning};
pub use hasher::hash_prompt;
pub use signer::{sign_prompt, verify_prompt};