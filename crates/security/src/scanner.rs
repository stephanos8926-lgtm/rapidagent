//! Security scanning for prompt injection vulnerabilities
//!
//! Detects potential injection patterns in compiled prompts.

use rapidagent_ir::PromptIR;

#[derive(Debug, Clone)]
pub struct SecurityWarning {
    pub severity: String,
    pub section: String,
    pub message: String,
}

/// Scan PromptIR for security issues
pub fn scan_for_injection(ir: &PromptIR) -> Vec<SecurityWarning> {
    let mut warnings = Vec::new();
    
    for section in &ir.sections {
        if section.content.contains("<script>") || section.content.contains("javascript:") {
            warnings.push(SecurityWarning {
                severity: "HIGH".to_string(),
                section: section.name.clone(),
                message: "Potential XSS injection detected".to_string(),
            });
        }
        
        if section.content.contains("${{") || section.content.contains("#{") {
            warnings.push(SecurityWarning {
                severity: "MEDIUM".to_string(),
                section: section.name.clone(),
                message: "Potential template injection detected".to_string(),
            });
        }
        
        if section.content.contains("exec(") || section.content.contains("eval(") {
            warnings.push(SecurityWarning {
                severity: "HIGH".to_string(),
                section: section.name.clone(),
                message: "Potential code execution injection detected".to_string(),
            });
        }
    }
    
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{Section, Priority};
    
    #[test]
    fn test_scan_safe_content() {
        let ir = PromptIR {
            name: "test".to_string(),
            version: rapidagent_ir::Semver::new(1, 0, 0),
            sections: vec![
                Section::new("identity", Priority::P1, "You are a helpful AI assistant."),
            ],
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        };
        
        let warnings = scan_for_injection(&ir);
        assert!(warnings.is_empty());
    }
    
    #[test]
    fn test_scan_dangerous_content() {
        let ir = PromptIR {
            name: "test".to_string(),
            version: rapidagent_ir::Semver::new(1, 0, 0),
            sections: vec![
                Section::new("identity", Priority::P1, "Execute <script>alert('xss')</script>"),
            ],
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        };
        
        let warnings = scan_for_injection(&ir);
        assert!(!warnings.is_empty());
        assert_eq!(warnings[0].severity, "HIGH");
    }
}