//! Markdown renderer for PromptIR
//!
//! Converts the intermediate representation to Markdown format.

use rapidagent_ir::PromptIR;
use std::fmt::Write;

/// Render PromptIR as Markdown
pub fn render_markdown(ir: &PromptIR) -> String {
    let mut output = String::new();
    
    writeln!(output, "# System Prompt: {}", ir.name).unwrap();
    writeln!(output).unwrap();
    writeln!(output, "**Version:** {}\n", ir.version).unwrap();
    
    for section in &ir.sections {
        writeln!(output, "## {}\n{}", section.name, section.content).unwrap();
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{Section, Priority};
    
    #[test]
    fn test_render_markdown() {
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
        
        let md = render_markdown(&ir);
        assert!(md.contains("## identity"));
        assert!(md.contains("You are an AI."));
    }
}