//! Graph validation for agent execution DAGs
//!
//! Validates structural integrity of compiled agent graphs.

use crate::types::{AgentGraph, Edge, Node, NodeType, ValidationError, ValidationCode};
use std::collections::{HashMap, HashSet};

/// Validate an agent graph for structural correctness
pub fn validate_graph(graph: &AgentGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check no duplicate node IDs
    errors.extend(check_duplicate_nodes(graph));

    // Check entry node exists
    errors.extend(check_entry_node(graph));

    // Check all edges reference valid nodes
    errors.extend(check_edge_targets(graph));

    // Check for cycles (except in Loop nodes which are allowed)
    errors.extend(check_cycles(graph));

    // Check reducer references
    errors.extend(check_reducer_references(graph));

    errors
}

fn check_duplicate_nodes(graph: &AgentGraph) -> Vec<ValidationError> {
    let mut ids = HashMap::new();
    let mut errors = Vec::new();

    for node in &graph.nodes {
        if ids.contains_key(&node.id) {
            errors.push(ValidationError {
                code: ValidationCode::DuplicateNode,
                message: format!("Node '{}' is duplicated", node.id),
                path: vec!["nodes".to_string(), node.id.clone()],
            });
        }
        ids.insert(node.id.clone(), node.name.clone());
    }

    errors
}

fn check_entry_node(graph: &AgentGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if !graph.nodes.iter().any(|n| n.id == graph.entry_node) {
        errors.push(ValidationError {
            code: ValidationCode::MissingEntry,
            message: format!("Entry node '{}' not found", graph.entry_node),
            path: vec!["entry_node".to_string()],
        });
    }

    errors
}

fn check_edge_targets(graph: &AgentGraph) -> Vec<ValidationError> {
    let node_ids: HashSet<&String> = graph.nodes.iter().map(|n| &n.id).collect();
    let mut errors = Vec::new();

    for edge in &graph.edges {
        if !node_ids.contains(&edge.from) {
            errors.push(ValidationError {
                code: ValidationCode::MissingNode,
                message: format!("Edge source '{}' not found", edge.from),
                path: vec!["edges".to_string(), edge.from.clone()],
            });
        }
        if !node_ids.contains(&edge.to) {
            errors.push(ValidationError {
                code: ValidationCode::MissingNode,
                message: format!("Edge target '{}' not found", edge.to),
                path: vec!["edges".to_string(), edge.to.clone()],
            });
        }
    }

    errors
}

fn check_cycles(graph: &AgentGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Build adjacency list excluding Loop->Join edges (which are cyclic by design)
    let adj: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        for edge in &graph.edges {
            // Don't consider edges involving Loop nodes as potential cycles
            let from_is_loop = graph.nodes.iter().any(|n| n.id == edge.from && matches!(n.node_type, NodeType::Loop));
            let to_is_join = graph.nodes.iter().any(|n| n.id == edge.to && matches!(n.node_type, NodeType::Join));
            
            if !from_is_loop || !to_is_join {
                map.entry(edge.from.clone()).or_insert_with(Vec::new).push(edge.to.clone());
            }
        }
        map
    };

    // DFS cycle detection
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    fn dfs(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    dfs(neighbor, adj, visited, rec_stack, path, errors);
                } else if rec_stack.contains(neighbor) {
                    // Found cycle
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let mut cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycle.push(neighbor.to_string());
                    
                    errors.push(ValidationError {
                        code: ValidationCode::CycleDetected,
                        message: format!("Cycle detected: {}", cycle.join(" -> ")),
                        path: cycle,
                    });
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            dfs(&node.id, &adj, &mut visited, &mut rec_stack, &mut Vec::new(), &mut errors);
        }
    }

    errors
}

fn check_reducer_references(graph: &AgentGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (node_id, _strategy) in &graph.reducers {
        if !graph.nodes.iter().any(|n| n.id == *node_id) {
            errors.push(ValidationError {
                code: ValidationCode::UnknownReducer,
                message: format!("Reducer references unknown node '{}'", node_id),
                path: vec!["reducers".to_string(), node_id.clone()],
            });
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, NodeType};

    #[test]
    fn test_validate_valid_graph() {
        let mut graph = AgentGraph::new("test", "Test graph", "n1");
        graph.add_node(Node::new("n1", "Node 1", NodeType::Agent));
        graph.add_node(Node::new("n2", "Node 2", NodeType::Tool));
        graph.add_edge(Edge::new("n1", "n2"));

        let errors = validate_graph(&graph);
        assert_eq!(errors.len(), 0, "Expected no validation errors");
    }

    #[test]
    fn test_validate_duplicate_nodes() {
        let mut graph = AgentGraph::new("test", "Test graph", "n1");
        graph.add_node(Node::new("n1", "Node 1", NodeType::Agent));
        graph.add_node(Node::new("n1", "Node 1 Dup", NodeType::Agent));

        let errors = validate_graph(&graph);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationCode::DuplicateNode);
    }

    #[test]
    fn test_validate_orphan_edge() {
        let mut graph = AgentGraph::new("test", "Test graph", "n1");
        graph.add_node(Node::new("n1", "Node 1", NodeType::Agent));
        graph.add_edge(Edge::new("n1", "nonexistent"));

        let errors = validate_graph(&graph);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationCode::MissingNode);
    }

    #[test]
    fn test_validate_missing_entry() {
        let mut graph = AgentGraph::new("test", "Test graph", "missing");
        graph.add_node(Node::new("n1", "Node 1", NodeType::Agent));

        let errors = validate_graph(&graph);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationCode::MissingEntry);
    }

    #[test]
    fn test_validate_cycle_detection() {
        let mut graph = AgentGraph::new("test", "Test graph", "n1");
        graph.add_node(Node::new("n1", "Node 1", NodeType::Agent));
        graph.add_node(Node::new("n2", "Node 2", NodeType::Tool));
        graph.add_node(Node::new("n3", "Node 3", NodeType::Tool));
        graph.add_edge(Edge::new("n1", "n2"));
        graph.add_edge(Edge::new("n2", "n3"));
        graph.add_edge(Edge::new("n3", "n1")); // Creates cycle

        let errors = validate_graph(&graph);
        assert!(errors.iter().any(|e| matches!(e.code, ValidationCode::CycleDetected)));
    }

    #[test]
    fn test_validate_loop_join_no_cycle() {
        let mut graph = AgentGraph::new("test", "Test graph", "n1");
        graph.add_node(Node::new("n1", "Entry", NodeType::Agent));
        graph.add_node(Node::new("loop", "Loop", NodeType::Loop));
        graph.add_node(Node::new("join", "Join", NodeType::Join));
        
        // Loop -> Join edge should not be flagged as cycle
        graph.add_edge(Edge::new("loop", "join"));

        let errors = validate_graph(&graph);
        assert!(!errors.iter().any(|e| matches!(e.code, ValidationCode::CycleDetected)));
    }
}
