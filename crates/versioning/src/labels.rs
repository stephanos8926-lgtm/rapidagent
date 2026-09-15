//! Label management for environment pointers
//!
//! Labels like "prod", "staging", "draft" point to specific versions.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Label pointing to a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,           // "prod", "staging", "draft"
    pub prompt_id: String,
    pub version: String,
    pub applied_at: DateTime<Utc>,
    pub applied_by: String,
}

/// Label store
#[derive(Debug, Default)]
pub struct LabelStore {
    labels: HashMap<String, HashMap<String, Label>>, // prompt_id -> label -> Label
}

impl LabelStore {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }
    
    pub fn set_label(
        &mut self,
        prompt_id: &str,
        label: &str,
        version: &str,
        author: &str,
    ) {
        let entry = Label {
            name: label.to_string(),
            prompt_id: prompt_id.to_string(),
            version: version.to_string(),
            applied_at: Utc::now(),
            applied_by: author.to_string(),
        };
        
        self.labels
            .entry(prompt_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(label.to_string(), entry);
    }
    
    pub fn get_label(&self, prompt_id: &str, label: &str) -> Option<&Label> {
        self.labels.get(prompt_id).and_then(|labels| labels.get(label))
    }
    
    pub fn list_labels(&self, prompt_id: &str) -> Vec<&Label> {
        self.labels.get(prompt_id)
            .map(|labels| labels.values().collect())
            .unwrap_or_default()
    }
    
    pub fn resolve(&self, prompt_id: &str, environment: &str) -> Option<String> {
        self.get_label(prompt_id, environment)
            .map(|l| l.version.clone())
    }
}
