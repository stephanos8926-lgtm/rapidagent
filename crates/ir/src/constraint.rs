//! Constraint types and validation

use crate::types::Priority;
use serde::{Deserialize, Serialize};

/// A constraint on prompt behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub text: String,
    pub priority: Priority,
    pub category: ConstraintCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintCategory {
    Safety,
    Format,
    Style,
    Tone,
    Content,
    Other,
}

impl std::fmt::Display for ConstraintCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintCategory::Safety => write!(f, "safety"),
            ConstraintCategory::Format => write!(f, "format"),
            ConstraintCategory::Style => write!(f, "style"),
            ConstraintCategory::Tone => write!(f, "tone"),
            ConstraintCategory::Content => write!(f, "content"),
            ConstraintCategory::Other => write!(f, "other"),
        }
    }
}

impl Constraint {
    pub fn new(id: &str, text: &str, priority: Priority) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            priority,
            category: ConstraintCategory::Other,
        }
    }
}
