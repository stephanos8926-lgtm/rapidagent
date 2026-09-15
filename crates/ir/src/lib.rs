//! System Prompt Compiler - Intermediate Representation
//!
//! This crate defines the typed AST/IR for system prompts and agent graphs,
//! supporting sections, priorities, constraints, state schemas, and graph validation.

pub mod types;
pub mod validation;

// Re-export core types for convenience
pub use types::{
    // Version
    Semver,
    // Priority
    Priority,
    // Prompt sections (legacy)
    Section,
    PromptIR,
    PromptMetadata,
    // Variables
    Variable,
    // Output schema
    OutputSchema,
    // Agent graph
    Node,
    NodeType,
    Edge,
    AgentGraph,
    GraphMetadata,
    // Artifact
    CompiledArtifact,
    ArtifactRef,
    ArtifactMetadata,
    // State schema
    StateSchema,
    FieldDef,
    FieldType,
    // Reducers
    ReduceStrategy,
    // Tool bindings
    ToolBinding,
    AgentConfig,
    RetryPolicy,
    // Validation
    ValidationError,
    ValidationCode,
};
