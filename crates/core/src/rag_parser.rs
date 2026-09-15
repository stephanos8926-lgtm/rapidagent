//! .rag format parser — YAML frontmatter + graph body
//!
//! The .rag format supports two modes:
//! 1. **Flat mode**: YAML frontmatter only + prompt sections (compatible with .spc)
//! 2. **Graph mode**: YAML frontmatter + <graph> block with nodes/edges

use rapidagent_ir::{Node, NodeType, Edge, Priority, AgentGraph, StateSchema};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RagParseError {
    #[error("Missing YAML frontmatter (---)")]
    MissingFrontmatter,
    #[error("Invalid YAML: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),
    #[error("Unknown node type: {0}")]
    UnknownNodeType(String),
    #[error("Missing required field '{0}' on node")]
    MissingNodeField(String),
    #[error("Parse error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RagParseError>;

/// YAML frontmatter extracted from .rag file
#[derive(Debug, Clone)]
pub struct RagFrontmatter {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
}

impl RagFrontmatter {
    fn from_yaml(s: &str) -> Result<Self> {
        let map: serde_yaml::Value = serde_yaml::from_str(s)?;
        let map = map.as_mapping().ok_or_else(|| {
            RagParseError::Other("Frontmatter must be a YAML mapping".to_string())
        })?;

        let name = map
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .to_string();

        let version = map.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
        let description = map
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let priority = map
            .get("priority")
            .and_then(|v| v.as_str())
            .map(|s| Priority::from_str(s).unwrap_or(Priority::P3));
        let tags = map
            .get("tags")
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(Self { name, version, description, priority, tags })
    }
}

/// Parsed body sections
#[derive(Debug, Clone)]
pub struct RagSection {
    pub name: String,
    pub priority: Priority,
    pub content: String,
}

/// A node declaration in graph block
#[derive(Debug, Clone)]
pub struct RagNode {
    pub id: String,
    pub node_type: NodeType,
    pub priority: Priority,
    pub config: serde_json::Value,
}

/// An edge declaration
#[derive(Debug, Clone)]
pub struct RagEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

/// A graph block
#[derive(Debug, Clone)]
pub struct RagGraph {
    pub nodes: Vec<RagNode>,
    pub edges: Vec<RagEdge>,
}

/// Complete parsed .rag artifact
#[derive(Debug, Clone)]
pub struct RagArtifact {
    pub frontmatter: RagFrontmatter,
    pub sections: Vec<RagSection>,
    pub graph: Option<RagGraph>,
}

/// Parse a .rag file
pub fn parse_rag(source: &str) -> Result<RagArtifact> {
    let (frontmatter_str, body) = extract_frontmatter(source)?;
    let frontmatter = RagFrontmatter::from_yaml(frontmatter_str)?;
    let (sections, graph) = parse_body(body)?;
    Ok(RagArtifact { frontmatter, sections, graph })
}

fn extract_frontmatter(source: &str) -> Result<(&str, &str)> {
    let trimmed = source.trim();
    if !trimmed.starts_with("---") {
        return Err(RagParseError::MissingFrontmatter);
    }
    let end = trimmed[3..].find("\n---").ok_or(RagParseError::MissingFrontmatter)? + 3;
    let frontmatter = &trimmed[3..end];
    let body = trimmed[end + 5..].trim();
    Ok((frontmatter.trim(), body))
}

fn parse_body(body: &str) -> Result<(Vec<RagSection>, Option<RagGraph>)> {
    let (sections, graph) = if let Some(range) = find_graph_block(body) {
        let graph_source = &body[range.clone()];
        let graph = Some(parse_graph_block(graph_source)?);
        let without_graph = format!("{}{}", &body[..range.start], &body[range.end..]);
        (parse_sections(&without_graph)?, graph)
    } else {
        (parse_sections(body)?, None)
    };
    Ok((sections, graph))
}

fn find_graph_block(body: &str) -> Option<std::ops::Range<usize>> {
    let bytes = body.as_bytes();
    let start_marker = b"<graph>";
    let end_marker = b"</graph>";
    for i in 0..bytes.len() {
        if &bytes[i..].get(..start_marker.len())? == start_marker {
            let after_start = i + start_marker.len();
            if let Some(end_pos) = bytes[after_start..].windows(end_marker.len()).position(|w| w == end_marker) {
                return Some(i..after_start + end_pos + end_marker.len());
            }
        }
    }
    None
}

fn parse_graph_block(block: &str) -> Result<RagGraph> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let re = regex::Regex::new(r#"<(node|edge)\s+([^>]*)/?>"#.trim_end()).unwrap();
    
    for cap in re.captures_iter(block) {
        let tag = cap[1].to_string();
        let attrs_str = &cap[2];
        let attrs = parse_kv_attrs(attrs_str)?;
        
        match tag.as_str() {
            "node" => {
                let id = attrs.get("id").cloned().ok_or_else(|| RagParseError::MissingNodeField("id".into()))?;
                let ntype = attrs.get("type").cloned().unwrap_or_else(|| "Agent".to_string());
                let pri_str = attrs.get("priority").cloned().unwrap_or_else(|| "P3".to_string());
                let node_type = match ntype.as_str() {
                    "Tool" => NodeType::Tool,
                    "Condition" => NodeType::Condition,
                    "Parallel" => NodeType::Parallel,
                    "Join" => NodeType::Join,
                    "Loop" => NodeType::Loop,
                    "Memory" => NodeType::Memory,
                    "Evaluator" => NodeType::Evaluator,
                    _ => NodeType::Agent,
                };
                let priority = Priority::from_str(&pri_str).unwrap_or(Priority::P3);
                let config: serde_json::Value = serde_json::from_value(serde_json::json!({})).unwrap();
                nodes.push(RagNode { id, node_type, priority, config });
            }
            "edge" => {
                let from = attrs.get("from").cloned().ok_or_else(|| RagParseError::MissingNodeField("from".into()))?;
                let to = attrs.get("to").cloned().ok_or_else(|| RagParseError::MissingNodeField("to".into()))?;
                let label = attrs.get("label").cloned().unwrap_or_else(|| "default".to_string());
                edges.push(RagEdge { from, to, label });
            }
            _ => {}
        }
    }
    Ok(RagGraph { nodes, edges })
}

fn parse_kv_attrs(s: &str) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    let re = regex::Regex::new(r#"(\w+)=(?:"([^"]*)"|'([^']*)')"#).unwrap();
    for cap in re.captures_iter(s) {
        let key = cap[1].to_string();
        let value = cap.get(2).map(|m| m.as_str()).or_else(|| cap.get(3).map(|m| m.as_str())).unwrap_or("");
        map.insert(key, value.to_string());
    }
    Ok(map)
}

fn parse_sections(body: &str) -> Result<Vec<RagSection>> {
    let mut sections = Vec::new();
    let mut remaining = body.trim();

    while let Some(start) = remaining.find('<') {
        let rest = &remaining[start + 1..];
        if let Some(end_tag) = rest.find('>') {
            let tag_line = &rest[..end_tag];
            let parts: Vec<&str> = tag_line.split_whitespace().collect();
            if parts.is_empty() {
                remaining = &rest[end_tag + 1..];
                continue;
            }
            let tag_name = parts[0];
            let priority = if let Some(pri_part) = parts.iter().find(|p| p.starts_with("priority=")) {
                let val = pri_part.strip_prefix("priority=").unwrap_or("");
                Priority::from_str(val).unwrap_or(Priority::P3)
            } else {
                Priority::P3
            };
            // Find closing tag
            let close_tag = format!("</{}", tag_name);
            if let Some(content_start) = rest.find('>') {
                let after_open = &rest[content_start + 1..];
                if let Some(end_pos) = after_open.find(&close_tag) {
                    let content = after_open[..end_pos].trim().to_string();
                    sections.push(RagSection {
                        name: tag_name.to_string(),
                        priority,
                        content,
                    });
                    remaining = &after_open[end_pos + close_tag.len()..];
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    Ok(sections)
}

impl RagArtifact {
    /// Convert to CompiledArtifact (for folder/binary output)
    pub fn to_compiled_artifact(&self) -> rapidagent_ir::CompiledArtifact {
        use rapidagent_ir::{CompiledArtifact, Semver, Section, ArtifactMetadata};
        use chrono::Utc;
        
        let version = Semver::new(1, 0, 0);
        
        // Build agent graph if present
        let graph = self.to_agent_graph().unwrap_or_else(|| {
            use rapidagent_ir::AgentGraph;
            AgentGraph::new(&self.frontmatter.name, "", "identity")
        });
        
        let mut artifact = CompiledArtifact::new(&self.frontmatter.name, version, graph);
        
        // Add prompt sections
        for section in &self.sections {
            artifact.add_section(Section::new(&section.name, section.priority.clone(), &section.content));
        }
        
        // Set metadata
        artifact.metadata = ArtifactMetadata {
            author: self.frontmatter.tags.first().cloned().unwrap_or_default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            uuid: artifact.compute_uuid(),
            hash: artifact.content_hash(),
            tags: self.frontmatter.tags.clone(),
        };
        
        artifact
    }

    /// Convert to PromptIR (legacy)
    pub fn to_prompt_ir(&self) -> rapidagent_ir::PromptIR {
        use rapidagent_ir::{Semver, PromptIR, Section};
        let version = Semver::new(1, 0, 0);
        let mut ir = PromptIR::new(&self.frontmatter.name, version);
        for section in &self.sections {
            ir.add_section(Section::new(&section.name, section.priority.clone(), &section.content));
        }
        ir
    }

    /// Convert to AgentGraph
    pub fn to_agent_graph(&self) -> Option<AgentGraph> {
        let graph = self.graph.as_ref()?;
        if graph.nodes.is_empty() {
            return None;
        }
        let entry = graph.nodes.first()?.id.clone();
        let mut ag = AgentGraph::new(&self.frontmatter.name, self.frontmatter.description.as_deref().unwrap_or(""), &entry);
        for node in &graph.nodes {
            let n = Node::new(&node.id, &node.id, node.node_type.clone());
            ag.add_node(n);
        }
        for edge in &graph.edges {
            let e = Edge::new(&edge.from, &edge.to);
            ag.add_edge(e);
        }
        Some(ag)
    }

    /// Validate graph
    pub fn validate_graph(&self) -> std::result::Result<AgentGraph, rapidagent_ir::ValidationError> {
        use rapidagent_ir::validation::validate_graph;
        let ag = self.to_agent_graph().ok_or_else(|| rapidagent_ir::ValidationError {
            code: rapidagent_ir::ValidationCode::MissingNode,
            message: "No graph section found".into(),
            path: vec!["graph".into()],
        })?;
        let errors = validate_graph(&ag);
        if errors.is_empty() {
            Ok(ag)
        } else {
            Err(errors[0].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flat_rag() {
        let source = "---\nname: test-agent\nversion: 1.0.0\n---\n\n<identity priority=\"P1\">\nYou are an AI assistant.\n</identity>\n<style priority=\"P2\">\nBe concise.\n</style>";
        let artifact = parse_rag(source).unwrap();
        assert_eq!(artifact.frontmatter.name, "test-agent");
        assert_eq!(artifact.frontmatter.version, Some("1.0.0".to_string()));
        assert_eq!(artifact.sections.len(), 2);
        assert_eq!(artifact.sections[0].name, "identity");
        assert!(artifact.graph.is_none());
    }

    #[test]
    fn test_parse_graph_rag() {
        let source = "---\nname: agent-with-graph\nversion: 2.0.0\n---\n\n<identity priority=\"P1\">You are an AI.</identity>\n\n<graph>\n  <node id=\"n1\" type=\"Agent\" priority=\"P1\" />\n  <node id=\"n2\" type=\"Tool\" priority=\"P2\" tool=\"echo\" />\n  <edge from=\"n1\" to=\"n2\" label=\"success\" />\n</graph>";
        let artifact = parse_rag(source).unwrap();
        assert_eq!(artifact.frontmatter.name, "agent-with-graph");
        assert!(artifact.graph.is_some());
        let graph = artifact.graph.as_ref().unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.nodes[0].id, "n1");
    }

    #[test]
    fn test_parse_invalid_no_frontmatter() {
        let source = "<identity priority=\"P1\">You are an AI.</identity>";
        assert!(parse_rag(source).is_err());
    }

    #[test]
    fn test_to_prompt_ir() {
        let source = "---\nname: test-agent\n---\n<identity priority=\"P1\">You are an AI.</identity>";
        let artifact = parse_rag(source).unwrap();
        let ir = artifact.to_prompt_ir();
        assert_eq!(ir.name, "test-agent");
        assert_eq!(ir.sections.len(), 1);
    }
}
