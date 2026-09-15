//! Parser for .spc token stream into PromptIR AST
//!
//! Converts tokens from the lexer into a structured PromptIR representation.

use super::lexer::{Token, Lexer};
use rapidagent_ir::{PromptIR, Section, Priority, Semver};
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Expected closing tag for '{0}', found '{1}'")]
    MismatchedTag(String, String),
    
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    
    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),
    
    #[error("Invalid priority: {0}")]
    InvalidPriority(String),
    
    #[error("Parser error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<PromptIR> {
        debug!("Starting parser");
        
        // Skip any leading comments
        while matches!(self.peek(), Some(Token::Comment(_))) {
            self.advance();
        }
        
        // Expect root <syspro> tag
        let (_name, attrs) = self.expect_tag_start("syspro")?;
        
        // Extract name from attributes
        let name = attrs.iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        
        let version = Semver::new(1, 0, 0);
        
        let mut sections = Vec::new();
        
        // Parse child sections until closing tag
        while !matches!(self.peek(), Some(Token::TagEnd(_))) {
            match self.peek() {
                Some(Token::Comment(_)) => {
                    self.advance();
                }
                _ => {
                    if let Some(section) = self.parse_section()? {
                        sections.push(section);
                    }
                }
            }
        }
        
        // Consume closing tag
        self.expect_tag_end("syspro")?;
        
        Ok(PromptIR {
            version,
            name,
            sections,
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        })
    }

    fn parse_section(&mut self) -> Result<Option<Section>> {
        let tag = match self.peek() {
            Some(Token::TagStart(name, attrs)) => {
                // Known section types
                if matches!(name.as_str(), "identity" | "style" | "protocols" | "constraints" | "context" | "variables") {
                    Some((name.clone(), attrs.clone()))
                } else {
                    None
                }
            }
            Some(Token::Comment(_)) => return Ok(None),
            _ => return Ok(None),
        };

        let (name, attrs) = tag.unwrap();
        
        // Extract priority
        let priority_str = attrs.iter()
            .find(|(k, _)| k == "priority")
            .map(|(_, v)| v.as_str())
            .unwrap_or("P3");
        
        let priority = self.parse_priority(priority_str)?;
        
        let mut content = String::new();
        
        // Parse content until closing tag
        loop {
            match self.peek() {
                Some(Token::TagEnd(n)) if n.as_str() == name.as_str() => {
                    self.advance();
                    break;
                }
                Some(Token::Text(t)) => {
                    content.push_str(&t);
                    content.push('\n');
                    self.advance();
                }
                Some(Token::Comment(_)) => {
                    self.advance();
                }
                Some(Token::Eof) => break,
                _ => {
                    self.advance();
                }
            }
        }
        
        Ok(Some(Section::new(&name, priority, &content)))
    }

    fn parse_priority(&self, s: &str) -> Result<Priority> {
        Priority::from_str(s).map_err(|e| ParseError::InvalidPriority(e))
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect_tag_start(&mut self, expected_name: &str) -> Result<(String, Vec<(String, String)>)> {
        match self.advance() {
            Some(Token::TagStart(name, attrs)) if name == expected_name => {
                Ok((name, attrs))
            }
            Some(other) => Err(ParseError::UnexpectedToken(other)),
            _ => Err(ParseError::Other(format!("Expected <{}>", expected_name))),
        }
    }

    fn expect_tag_end(&mut self, expected_name: &str) -> Result<()> {
        match self.advance() {
            Some(Token::TagEnd(name)) if name == expected_name => Ok(()),
            Some(other) => Err(ParseError::MismatchedTag(expected_name.to_string(), 
                match &other {
                    Token::TagEnd(n) => n.clone(),
                    _ => format!("{:?}", other),
                })),
            _ => Err(ParseError::Other(format!("Expected </{}>", expected_name))),
        }
    }
}

/// Parse tokens into PromptIR (convenience function)
pub fn parse_tokens(tokens: Vec<Token>) -> Result<PromptIR> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Convenience function: read source, lex, parse
pub fn compile_source(source: &str) -> Result<PromptIR> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().to_vec();
    parse_tokens(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let source = r#"<syspro version="1.0.0" name="test">
  <identity priority="P1">
    You are an AI assistant.
  </identity>
</syspro>"#;
        
        let ir = compile_source(source).unwrap();
        assert_eq!(ir.name, "test");
        assert_eq!(ir.version, Semver::new(1, 0, 0));
        assert_eq!(ir.sections.len(), 1);
        assert_eq!(ir.sections[0].name, "identity");
    }

    #[test]
    fn test_parse_multiple_sections() {
        let source = r#"<syspro name="multi">
  <identity priority="P1">Be helpful</identity>
  <style priority="P2">Be concise</style>
</syspro>"#;
        
        let ir = compile_source(source).unwrap();
        assert_eq!(ir.sections.len(), 2);
        assert_eq!(ir.sections[0].name, "identity");
        assert_eq!(ir.sections[1].name, "style");
    }
}