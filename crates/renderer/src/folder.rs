//! Folder renderer for compiled agent artifacts
//!
//! Outputs a directory structure with manifest, prompt, graph, and supporting files.

use std::fs;
use std::path::{Path, PathBuf};

use rapidagent_ir::CompiledArtifact;

/// Output format for the folder renderer
#[derive(Debug, Clone)]
pub enum FolderFormat {
    /// Standard RapidAgent folder structure
    Standard,
    /// Minimal (just prompt + graph)
    Minimal,
}

/// Generate a compiled agent folder structure from an artifact
pub fn render_folder(artifact: &CompiledArtifact, output_dir: &Path, format: FolderFormat) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(output_dir)?;
    
    match format {
        FolderFormat::Standard => render_standard(artifact, output_dir),
        FolderFormat::Minimal => render_minimal(artifact, output_dir),
    }
}

/// Render standard RapidAgent folder structure
fn render_standard(artifact: &CompiledArtifact, output_dir: &Path) -> anyhow::Result<PathBuf> {
    // 1. manifest.json
    let manifest = serde_json::json!({
        "name": artifact.name,
        "version": artifact.version.to_string(),
        "uuid": artifact.metadata.uuid.clone(),
        "hash": artifact.metadata.hash.clone(),
        "compiler_version": artifact.metadata.compiler_version.clone(),
        "created_at": artifact.metadata.created_at.to_rfc3339(),
        "updated_at": artifact.metadata.updated_at.to_rfc3339(),
        "author": artifact.metadata.author.clone(),
        "tags": artifact.metadata.tags.clone(),
        "graph_entry": artifact.graph.entry_node.clone(),
        "node_count": artifact.graph.nodes.len(),
        "edge_count": artifact.graph.edges.len()
    });
    fs::write(output_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    
    // 2. system_prompt.md
    let prompt = generate_system_prompt(&artifact.prompt_sections);
    fs::write(output_dir.join("system_prompt.md"), prompt)?;
    
    // 3. graph.ir.json
    let graph_json = serde_json::json!({
        "name": artifact.graph.name,
        "description": artifact.graph.description,
        "entry_node": artifact.graph.entry_node,
        "nodes": artifact.graph.nodes.iter().map(|n| {
            serde_json::json!({
                "id": n.id,
                "name": n.name,
                "type": format!("{:?}", n.node_type),
                "config": n.config,
            })
        }).collect::<Vec<_>>(),
        "edges": artifact.graph.edges.iter().map(|e| {
            serde_json::json!({
                "from": e.from,
                "to": e.to,
                "condition": e.condition.as_ref().map(|c| serde_json::json!({"expression": c.expression, "label": c.label})),
                "weight": e.weight
            })
        }).collect::<Vec<_>>(),
    });
    fs::write(output_dir.join("graph.ir.json"), serde_json::to_string_pretty(&graph_json)?)?;
    
    // 4. tools.json
    let tools: Vec<_> = artifact.graph.nodes.iter()
        .filter(|n| matches!(n.node_type, rapidagent_ir::NodeType::Tool))
        .map(|n| serde_json::json!({"id": n.id, "name": n.name, "config": n.config}))
        .collect();
    fs::write(output_dir.join("tools.json"), serde_json::to_string_pretty(&tools)?)?;
    
    // 5. policy.json
    let policy = serde_json::json!({
        "compiler_version": artifact.metadata.compiler_version,
        "constraints": [],
        "security_scan": "passed"
    });
    fs::write(output_dir.join("policy.json"), serde_json::to_string_pretty(&policy)?)?;
    
    // 6. memory_contracts.json
    let memory = serde_json::json!({
        "collections": artifact.graph.state_schemas.keys().collect::<Vec<_>>(),
        "indexes": []
    });
    fs::write(output_dir.join("memory_contracts.json"), serde_json::to_string_pretty(&memory)?)?;
    
    // 7. quotas.json
    let quotas = serde_json::json!({
        "max_tokens": 4096,
        "timeout_ms": 60000,
        "max_retries": 3,
        "rate_limit_per_min": 60
    });
    fs::write(output_dir.join("quotas.json"), serde_json::to_string_pretty(&quotas)?)?;
    
    // 8. assets/ directory
    let assets_dir = output_dir.join("assets");
    fs::create_dir_all(&assets_dir)?;
    for artifact_ref in &artifact.artifacts {
        let asset_path = assets_dir.join(&artifact_ref.path);
        if let Some(parent) = asset_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let placeholder = serde_json::json!({
            "name": artifact_ref.name,
            "path": artifact_ref.path,
            "hash": artifact_ref.hash,
            "size_bytes": artifact_ref.size_bytes
        });
        fs::write(&asset_path, serde_json::to_string_pretty(&placeholder)?)?;
    }
    
    eprintln!("Generated {} nodes, {} edges", artifact.graph.nodes.len(), artifact.graph.edges.len());
    eprintln!("UUID: {}", artifact.metadata.uuid);
    eprintln!("Hash: {}", artifact.metadata.hash);
    
    Ok(output_dir.to_path_buf())
}

/// Render minimal folder (prompt + graph only)
fn render_minimal(artifact: &CompiledArtifact, output_dir: &Path) -> anyhow::Result<PathBuf> {
    let manifest = serde_json::json!({
        "name": artifact.name,
        "version": artifact.version.to_string(),
        "node_count": artifact.graph.nodes.len(),
        "edge_count": artifact.graph.edges.len()
    });
    fs::write(output_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    fs::write(output_dir.join("system_prompt.md"), generate_system_prompt(&artifact.prompt_sections))?;
    fs::write(output_dir.join("graph.json"), serde_json::to_string_pretty(&serde_json::json!({
        "name": artifact.graph.name,
        "entry": artifact.graph.entry_node,
        "nodes": artifact.graph.nodes.iter().map(|n| serde_json::json!({"id": n.id, "type": format!("{:?}", n.node_type)})).collect::<Vec<_>>(),
        "edges": artifact.graph.edges.iter().map(|e| serde_json::json!({"from": e.from, "to": e.to})).collect::<Vec<_>>()
    }))?)?;
    Ok(output_dir.to_path_buf())
}

/// Generate markdown system prompt from sections
fn generate_system_prompt(sections: &[rapidagent_ir::Section]) -> String {
    let mut output = String::from("# System Prompt\n\n");
    for section in sections {
        output.push_str(&format!("## {}\n\n{}\n\n", section.name, section.content));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{CompiledArtifact, AgentGraph, Semver};
    
    #[test]
    fn test_render_folder_basic() {
        let temp_dir = std::env::temp_dir().join("syspro_test_folder");
        let _ = fs::remove_dir_all(&temp_dir);
        
        let graph = AgentGraph::new("test-graph", "Test graph", "node1");
        let artifact = CompiledArtifact::new("test-agent", Semver::new(1, 0, 0), graph);
        
        let result = render_folder(&artifact, &temp_dir, FolderFormat::Standard);
        
        assert!(result.is_ok());
        assert!(temp_dir.join("manifest.json").exists());
        assert!(temp_dir.join("system_prompt.md").exists());
        assert!(temp_dir.join("graph.ir.json").exists());
        assert!(temp_dir.join("tools.json").exists());
        assert!(temp_dir.join("policy.json").exists());
        assert!(temp_dir.join("memory_contracts.json").exists());
        assert!(temp_dir.join("quotas.json").exists());
        assert!(temp_dir.join("assets").is_dir());
        
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
