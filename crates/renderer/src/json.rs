//! JSON renderer for PromptIR
//!
//! Converts the intermediate representation to JSON format.

use rapidagent_ir::PromptIR;

/// Render PromptIR as JSON
pub fn render_json(ir: &PromptIR) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(ir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{Section, Priority};
    
    #[test]
    fn test_render_json() {
        let ir = PromptIR {
            name: "test".to_string(),
            version: rapidagent_ir::Semver::new(1, 0, 0),
            sections: vec![
                Section::new("identity", Priority::P1, "You are an AI."),
            ],
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        };
        
        let json = render_json(&ir).unwrap();
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"identity\""));
    }
}