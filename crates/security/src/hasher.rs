//! Content hashing for version control
//!
//! Generates SHA-256 hashes for prompt content.

use rapidagent_ir::PromptIR;

/// Hash the prompt content for version tracking
pub fn hash_prompt(ir: &PromptIR) -> String {
    ir.content_hash()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{Section, Priority};
    
    #[test]
    fn test_hash_deterministic() {
        let ir1 = PromptIR {
            name: "test".to_string(),
            version: rapidagent_ir::Semver::new(1, 0, 0),
            sections: vec![
                Section::new("identity", Priority::P1, "You are an AI."),
            ],
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        };
        
        let ir2 = ir1.clone();
        
        assert_eq!(hash_prompt(&ir1), hash_prompt(&ir2));
    }
    
    #[test]
    fn test_hash_different_content() {
        let ir1 = PromptIR {
            name: "test".to_string(),
            version: rapidagent_ir::Semver::new(1, 0, 0),
            sections: vec![
                Section::new("identity", Priority::P1, "You are AI-1."),
            ],
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        };
        
        let mut ir2 = ir1.clone();
        ir2.sections[0].content = "You are AI-2.".to_string();
        
        assert_ne!(hash_prompt(&ir1), hash_prompt(&ir2));
    }
}