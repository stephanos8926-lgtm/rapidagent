//! Section management and operations

use crate::types::Section;
use std::collections::HashMap;

impl Section {
    /// Check if this section should be loaded based on condition
    pub fn should_load(&self, context: &HashMap<String, bool>) -> bool {
        match &self.condition {
            None => true,
            Some(cond) => {
                // Simple condition evaluation
                // Format: "feature_x=true" or "budget_exceeded=false"
                if let Some((key, expected)) = cond.split_once('=') {
                    let key = key.trim();
                    let expected = expected.trim().parse::<bool>().unwrap_or(false);
                    context.get(key).copied().unwrap_or(expected)
                } else {
                    true // No condition specified, load always
                }
            }
        }
    }

    /// Get section content with variable substitution
    pub fn render(&self, variables: &HashMap<String, String>) -> String {
        let mut result = self.content.clone();
        for (name, value) in variables {
            result = result.replace(&format!("{{{}}}", name), value);
        }
        result
    }
}
