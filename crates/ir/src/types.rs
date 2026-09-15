//! Core IR types for the System Prompt Compiler
//!
//! Defines the complete Intermediate Representation for agent artifacts,
//! including graph nodes, edges, state schema, and validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// VERSION
// ============================================================================

/// Semantic version for prompt versions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for Semver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Semver {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

// ============================================================================
// PRIORITY
// ============================================================================

/// Priority label (P1-P3) with numeric weight
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    P1, // Critical - identity, safety
    P2, // High - protocols, rules
    P3, // Medium - context, hints
    Custom(u8), // 0-100 numeric weight
}

impl Priority {
    /// Convert to numeric weight (100 = highest)
    pub fn weight(&self) -> u8 {
        match self {
            Priority::P1 => 100,
            Priority::P2 => 80,
            Priority::P3 => 60,
            Priority::Custom(w) => *w,
        }
    }

    /// Parse from string (handles both "P1" and "85")
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "P1" | "CRITICAL" | "HIGH" => Ok(Priority::P1),
            "P2" | "MEDIUM" | "NORMAL" => Ok(Priority::P2),
            "P3" | "LOW" => Ok(Priority::P3),
            _ => {
                match s.parse::<u8>() {
                    Ok(w) if w <= 100 => Ok(Priority::Custom(w)),
                    Ok(w) => Err(format!("Weight {} out of range 0-100", w)),
                    Err(_) => Err(format!("Unknown priority: {}", s)),
                }
            }
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
            Priority::P3 => write!(f, "P3"),
            Priority::Custom(w) => write!(f, "{}", w),
        }
    }
}

// ============================================================================
// PROMPT SECTIONS (legacy, kept for compatibility)
// ============================================================================

/// A single section within a system prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub name: String,
    pub priority: Priority,
    pub content: String,
    pub condition: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Section {
    pub fn new(name: &str, priority: Priority, content: &str) -> Self {
        Self {
            id: format!("section_{}", name.replace('-', "_")),
            name: name.to_string(),
            priority,
            content: content.to_string(),
            condition: None,
            metadata: HashMap::new(),
        }
    }
}

/// Prompt metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub changelog: String,
    pub tags: Vec<String>,
}

impl Default for PromptMetadata {
    fn default() -> Self {
        Self {
            author: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            changelog: String::new(),
            tags: Vec::new(),
        }
    }
}

/// Complete system prompt IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptIR {
    pub version: Semver,
    pub name: String,
    pub sections: Vec<Section>,
    pub variables: Vec<Variable>,
    pub output_schema: Option<OutputSchema>,
    pub metadata: PromptMetadata,
}

impl PromptIR {
    pub fn new(name: &str, version: Semver) -> Self {
        Self {
            version,
            name: name.to_string(),
            sections: Vec::new(),
            variables: Vec::new(),
            output_schema: None,
            metadata: PromptMetadata::default(),
        }
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
        // Sort by priority (highest first)
        self.sections.sort_by(|a, b| b.priority.weight().cmp(&a.priority.weight()));
    }

    /// Compute content hash for versioning
    pub fn content_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(self).unwrap());
        hex::encode(hasher.finalize())
    }
}

// ============================================================================
// GRAPH NODES
// ============================================================================

/// Node type in the agent execution graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Agent node: calls LLM with tool bindings
    Agent,
    /// Tool call node: invokes a specific tool
    Tool,
    /// Conditional: routes based on condition expression
    Condition,
    /// Parallel: fan-out to multiple branches
    Parallel,
    /// Join: merges parallel branches
    Join,
    /// Loop: iterates until condition met
    Loop,
    /// Memory: reads/writes to persistent store
    Memory,
    /// Evaluator: assesses output quality
    Evaluator,
}

/// Agent node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub system_prompt: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Vec<ToolBinding>,
    pub retry: RetryPolicy,
    pub timeout_ms: Option<u64>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: None,
            model: None,
            max_tokens: Some(2048),
            temperature: Some(0.7),
            tools: Vec::new(),
            retry: RetryPolicy::default(),
            timeout_ms: Some(30000),
        }
    }
}

/// Tool binding for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBinding {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
    pub handler: String, // module.path::function_name
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
    pub on_errors: Vec<String>, // error types to retry on
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
            on_errors: vec!["RateLimitError".to_string(), "TimeoutError".to_string()],
        }
    }
}

/// A node in the execution DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub config: serde_json::Value,
    pub input_schema: Option<StateSchema>,
    pub output_schema: Option<StateSchema>,
    pub metadata: HashMap<String, String>,
}

impl Node {
    pub fn new(id: &str, name: &str, node_type: NodeType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            node_type,
            config: serde_json::Value::Object(serde_json::Map::new()),
            input_schema: None,
            output_schema: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }

    pub fn with_input_schema(mut self, schema: StateSchema) -> Self {
        self.input_schema = Some(schema);
        self
    }

    pub fn with_output_schema(mut self, schema: StateSchema) -> Self {
        self.output_schema = Some(schema);
        self
    }
}

// ============================================================================
// GRAPH EDGES
// ============================================================================

/// Edge condition expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCondition {
    pub expression: String,
    pub label: String,
}

/// Edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String, // node ID
    pub to: String,   // node ID
    pub condition: Option<EdgeCondition>,
    pub weight: f64,
}

impl Edge {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            condition: None,
            weight: 1.0,
        }
    }

    pub fn with_condition(mut self, expr: &str, label: &str) -> Self {
        self.condition = Some(EdgeCondition {
            expression: expr.to_string(),
            label: label.to_string(),
        });
        self
    }
}

// ============================================================================
// STATE SCHEMA
// ============================================================================

/// Field type in a state schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FieldType {
    String,
    Int,
    Float,
    Bool,
    List(String),       // List<type>
    Map(String, String), // Map<key, value>
    Any,
}

/// Field definition in a state schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

/// State schema for input/output validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSchema {
    pub name: String,
    pub description: String,
    pub fields: Vec<FieldDef>,
}

impl StateSchema {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            fields: Vec::new(),
        }
    }

    pub fn add_field(mut self, name: &str, field_type: FieldType, description: &str, required: bool) -> Self {
        self.fields.push(FieldDef {
            name: name.to_string(),
            field_type,
            description: description.to_string(),
            required,
            default: None,
        });
        self
    }
}

// ============================================================================
// REDUCERS
// ============================================================================

/// Strategy for merging state from parallel branches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReduceStrategy {
    /// Last writer wins
    LastWriteWins,
    /// Merge maps recursively
    MergeMaps,
    /// Concatenate lists
    ConcatLists,
    /// Take first non-null value
    FirstNonNull,
    /// Custom reducer function
    Custom(String),
}

// ============================================================================
// AGENT GRAPH
// ============================================================================

/// The complete agent execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGraph {
    pub name: String,
    pub description: String,
    pub entry_node: String,    // ID of entry point
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub reducers: HashMap<String, ReduceStrategy>, // node_id -> strategy
    pub state_schemas: HashMap<String, StateSchema>, // name -> schema
    pub metadata: GraphMetadata,
}

impl AgentGraph {
    pub fn new(name: &str, description: &str, entry_node: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            entry_node: entry_node.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            reducers: HashMap::new(),
            state_schemas: HashMap::new(),
            metadata: GraphMetadata::default(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> &mut Self {
        self.nodes.push(node);
        self
    }

    pub fn add_edge(&mut self, edge: Edge) -> &mut Self {
        self.edges.push(edge);
        self
    }

    pub fn add_reducer(&mut self, node_id: &str, strategy: ReduceStrategy) -> &mut Self {
        self.reducers.insert(node_id.to_string(), strategy);
        self
    }

    pub fn add_schema(&mut self, name: &str, schema: StateSchema) -> &mut Self {
        self.state_schemas.insert(name.to_string(), schema);
        self
    }

    /// Compute UUID v5 from namespace + graph name
    pub fn compute_uuid(&self) -> String {
        // UUID v5 uses SHA-1, not SHA-256
        use sha1::{Sha1, Digest};
        let mut hasher = Sha1::new();
        hasher.update(self.name.as_bytes());
        let hash = hasher.finalize();
        // Use first 16 bytes to form UUID
        let bytes: [u8; 16] = hash[..16].try_into().unwrap();
        let uuid = Uuid::from_bytes(bytes);
        // Set version to 5 and variant to RFC4122
        let mut bytes = uuid.as_bytes().to_vec();
        bytes[6] = (bytes[6] & 0x0f) | 0x50; // version 5
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant
        Uuid::from_slice(&bytes).unwrap().to_string()
    }
}

/// Graph-level metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub compiler_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: String,
    pub tags: Vec<String>,
    pub changelog: String,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self {
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: String::new(),
            tags: Vec::new(),
            changelog: String::new(),
        }
    }
}

// ============================================================================
// MAIN ARTIFACT
// ============================================================================

/// Complete compiled agent artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledArtifact {
    pub name: String,
    pub version: Semver,
    pub graph: AgentGraph,
    pub prompt_sections: Vec<Section>,
    pub variables: Vec<Variable>,
    pub artifacts: Vec<ArtifactRef>,
    pub metadata: ArtifactMetadata,
}

impl CompiledArtifact {
    pub fn new(name: &str, version: Semver, graph: AgentGraph) -> Self {
        Self {
            name: name.to_string(),
            version,
            graph,
            prompt_sections: Vec::new(),
            variables: Vec::new(),
            artifacts: Vec::new(),
            metadata: ArtifactMetadata::default(),
        }
    }

    pub fn add_section(&mut self, section: Section) {
        self.prompt_sections.push(section);
    }

    /// Compute content hash for versioning
    pub fn content_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(self).unwrap());
        hex::encode(hasher.finalize())
    }

    /// Compute UUID v5 for provenance
    pub fn compute_uuid(&self) -> String {
        self.graph.compute_uuid()
    }
}

/// Reference to an artifact file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub name: String,
    pub path: String,
    pub hash: String,
    pub size_bytes: u64,
}

/// Top-level artifact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub compiler_version: String,
    pub uuid: String,
    pub hash: String,
    pub tags: Vec<String>,
}

impl Default for ArtifactMetadata {
    fn default() -> Self {
        Self {
            author: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            uuid: String::new(),
            hash: String::new(),
            tags: Vec::new(),
        }
    }
}

// ============================================================================
// VARIABLES
// ============================================================================

/// Variable placeholder for runtime substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    pub required: bool,
    #[serde(rename = "type")]
    pub r#type: String,
}

// ============================================================================
// OUTPUT SCHEMA
// ============================================================================

/// Output schema declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    pub format: String,
    pub constraints: Vec<String>,
    pub examples: Vec<String>,
}

// ============================================================================
// VALIDATION ERRORS
// ============================================================================

/// Validation error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: ValidationCode,
    pub message: String,
    pub path: Vec<String>,
}

/// Validation error codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationCode {
    MissingNode,
    DuplicateNode,
    OrphanEdge,
    CycleDetected,
    MissingEntry,
    InvalidSchema,
    UnknownReducer,
}

impl std::fmt::Display for ValidationCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationCode::MissingNode => write!(f, "MISSING_NODE"),
            ValidationCode::DuplicateNode => write!(f, "DUPLICATE_NODE"),
            ValidationCode::OrphanEdge => write!(f, "ORPHAN_EDGE"),
            ValidationCode::CycleDetected => write!(f, "CYCLE_DETECTED"),
            ValidationCode::MissingEntry => write!(f, "MISSING_ENTRY"),
            ValidationCode::InvalidSchema => write!(f, "INVALID_SCHEMA"),
            ValidationCode::UnknownReducer => write!(f, "UNKNOWN_REDUCER"),
        }
    }
}

impl std::error::Error for ValidationError {}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.path.join("."), self.message)
    }
}
