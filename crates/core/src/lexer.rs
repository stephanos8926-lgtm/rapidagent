//! Lexer for .spc prompt compiler source files
//!
//! Converts raw text into a stream of tokens for the parser.

use thiserror::Error;
use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `<tag attr="value">` or `</tag>`
    TagStart(String, Vec<(String, String)>),
    /// `</tag>`
    TagEnd(String),
    /// Text content between tags
    Text(String),
    /// `# comment` or `<!-- comment -->`
    Comment(String),
    /// End of file
    Eof,
}

#[derive(Debug, Error, Clone)]
pub enum LexError {
    #[error("Unexpected end of input while parsing tag")]
    UnexpectedEof,
    
    #[error("Invalid tag format: {0}")]
    InvalidTag(String),
    
    #[error("Unterminated string literal: {0}")]
    UnterminatedString(String),
}

pub type Result<T> = std::result::Result<T, LexError>;

#[derive(Debug)]
pub struct Lexer {
    source: String,
    pos: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            pos: 0,
            tokens: Vec::new(),
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> &[Token] {
        while self.pos < self.source.len() {
            self.skip_whitespace();
            if self.pos >= self.source.len() {
                break;
            }
            
            let c = self.peek().unwrap();
            match c {
                '<' => self.parse_tag_or_comment(),
                '#' => self.parse_comment(),
                _ => self.parse_text(),
            }
        }
        
        self.tokens.push(Token::Eof);
        &self.tokens
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.source.len() {
            let c = self.source[self.pos..].chars().next()?;
            self.pos += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            if self.source[self.pos..].starts_with([' ', '\t', '\n', '\r']) {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_tag_or_comment(&mut self) {
        debug!("Parsing tag or comment at pos {}", self.pos);
        
        if !self.source[self.pos..].starts_with('<') {
            return;
        }
        self.pos += 1; // skip '<'
        
        // Check for XML comment <!-- ... -->
        if self.source[self.pos..].starts_with("!--") {
            self.parse_xml_comment();
            return;
        }
        
        // Check for closing tag
        if self.peek() == Some('/') {
            self.pos += 1;
            let name = self.read_name();
            self.skip_whitespace();
            if self.peek() == Some('>') {
                self.pos += 1;
                self.tokens.push(Token::TagEnd(name));
            }
            return;
        }
        
        // Parse opening tag
        let name = self.read_name();
        let mut attrs = Vec::new();
        
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('>') => {
                    self.pos += 1;
                    self.tokens.push(Token::TagStart(name.clone(), attrs.clone()));
                    return;
                }
                Some('/') => {
                    self.pos += 1;
                    if self.peek() == Some('>') {
                        self.pos += 1;
                    }
                    self.tokens.push(Token::TagStart(name.clone(), attrs.clone()));
                    return;
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let attr_name = self.read_name();
                    self.skip_whitespace();
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        self.skip_whitespace();
                        let value = self.read_string_value();
                        attrs.push((attr_name, value));
                    } else {
                        attrs.push((attr_name, String::new()));
                    }
                }
                _ => {
                    self.tokens.push(Token::TagStart(name.clone(), attrs.clone()));
                    return;
                }
            }
        }
    }

    fn parse_xml_comment(&mut self) {
        debug!("Parsing XML comment at pos {}", self.pos);
        
        // Skip <!--
        self.pos += 3;
        
        let mut comment = String::new();
        while self.pos < self.source.len() {
            // Check for -->
            if self.source[self.pos..].starts_with("-->") {
                self.pos += 3;
                break;
            }
            comment.push(self.source[self.pos..].chars().next().unwrap());
            self.pos += 1;
        }
        
        self.tokens.push(Token::Comment(comment.trim().to_string()));
    }

    fn read_name(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        name
    }

    fn read_string_value(&mut self) -> String {
        self.skip_whitespace();
        
        let quote = match self.peek() {
            Some('"') | Some('\'') => self.advance().unwrap(),
            _ => return String::new(),
        };
        
        let mut value = String::new();
        while let Some(c) = self.peek() {
            if c == quote {
                self.advance();
                return value;
            }
            value.push(c);
            self.advance();
        }
        
        value
    }

    fn parse_comment(&mut self) {
        debug!("Parsing comment at pos {}", self.pos);
        
        self.pos += 1; // skip '#'
        let mut comment = String::new();
        
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                break;
            }
            comment.push(c);
            self.advance();
        }
        
        self.tokens.push(Token::Comment(comment.trim().to_string()));
    }

    fn parse_text(&mut self) {
        debug!("Parsing text at pos {}", self.pos);
        
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c == '<' || c == '#' {
                break;
            }
            text.push(c);
            self.advance();
        }
        
        if !text.trim().is_empty() {
            self.tokens.push(Token::Text(text.trim().to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let mut lexer = Lexer::new("<identity priority=\"P1\">");
        let tokens = lexer.tokenize();
        
        assert!(tokens.len() >= 2);
        assert_eq!(tokens[0], Token::TagStart("identity".to_string(), vec![("priority".to_string(), "P1".to_string())]));
    }

    #[test]
    fn test_closing_tag() {
        let mut lexer = Lexer::new("</identity>");
        let tokens = lexer.tokenize();
        
        assert!(tokens.len() >= 2);
        assert_eq!(tokens[0], Token::TagEnd("identity".to_string()));
    }

    #[test]
    fn test_text_content() {
        let mut lexer = Lexer::new("You are an AI assistant.");
        let tokens = lexer.tokenize();
        
        assert!(tokens.len() >= 2);
        assert_eq!(tokens[0], Token::Text("You are an AI assistant.".to_string()));
    }

    #[test]
    fn test_xml_comment() {
        let mut lexer = Lexer::new("<!-- This is a comment -->");
        let tokens = lexer.tokenize();
        
        assert!(tokens.len() >= 2);
        assert_eq!(tokens[0], Token::Comment("This is a comment".to_string()));
    }

    #[test]
    fn test_hash_comment() {
        let mut lexer = Lexer::new("# This is a hash comment");
        let tokens = lexer.tokenize();
        
        assert!(tokens.len() >= 2);
        assert_eq!(tokens[0], Token::Comment("This is a hash comment".to_string()));
    }
}