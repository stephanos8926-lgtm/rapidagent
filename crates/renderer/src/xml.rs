//! XML renderer for PromptIR
//!
//! Converts the intermediate representation back to human-readable XML.

use rapidagent_ir::PromptIR;
use std::fmt::Write;

/// Render PromptIR as XML
pub fn render_xml(ir: &PromptIR) -> String {
    let mut output = String::new();
    
    writeln!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>").unwrap();
    writeln!(output, "<syspro name=\"{}\" version=\"{}\">", ir.name, ir.version).unwrap();
    
    for section in &ir.sections {
        writeln!(output, "  <{}>", section.name).unwrap();
        if !section.content.is_empty() {
            for line in section.content.lines() {
                if !line.trim().is_empty() {
                    writeln!(output, "    {}", line).unwrap();
                }
            }
        }
        writeln!(output, "  </{}>", section.name).unwrap();
    }
    
    writeln!(output, "</syspro>").unwrap();
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{Section, Priority};
    
    #[test]
    fn test_render_xml() {
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
        
        let xml = render_xml(&ir);
        assert!(xml.contains("<syspro"));
        assert!(xml.contains("identity"));
        assert!(xml.contains("You are an AI."));
    }
}