# SPEC-004: Graph Execution Engine Integration

---
name: Graph-Execution-Engine
description: Integration of ri-agent-graph for agent workflow execution with cryptographic receipts
status: draft
created: 2026-09-16
author: Lucien
related-adrs:
  - "ADR-005"
---

# Spec: Graph Execution Engine

## Goal
Integrate `ri-agent-graph` as the primary execution engine for RapidAgent, providing DAG-based workflow execution with cryptographic execution receipts and SQLite checkpointing.

## Context
Phase 2 research identified `ri-agent-graph` as the optimal execution engine due to:
- Agent-specific node types (llm, router, join, parallel, human_approval)
- HMAC-SHA256 cryptographic receipts for execution integrity
- SQLite checkpointing with crash recovery
- JoinSet-backed parallel fan-out with 5 join modes

## Requirements

### R1: Core Engine Integration
The system SHALL integrate `ri-agent-graph` v0.2+ as the execution backend:

```toml
[dependencies]
ri-agent-graph = "0.2"
```

**Node Types Supported:**
| Node Type | Description | Input Schema | Output Schema |
|-----------|-------------|--------------|---------------|
| `llm` | LLM call node | `{prompt, model, params}` | `{response, tokens, latency}` |
| `router` | Conditional branching | `{input, conditions}` | `{next_node, condition_met}` |
| `join` | Synchronization point | `{inputs[], strategy}` | `{merged_result}` |
| `parallel` | Fan-out execution | `{tasks[], mode}` | `{results[], mode}` |
| `human_approval` | Human-in-the-loop | `{question, context}` | `{approved, response}` |

**Join Modes:**
- `all` — Wait for all inputs
- `first` — Return first completed
- `majority` — Return after majority complete
- `quorum` — Return after N quorum
- `timeout` — Return after timeout or all

### R2: Cryptographic Receipts
Every execution MUST produce an HMAC-SHA256 receipt:

```rust
pub struct ExecutionReceipt {
    pub node_id: String,
    pub input_hash: String,      // SHA256 of input
    pub output_hash: String,     // SHA256 of output
    pub hmac: String,            // HMAC-SHA256(input_hash || output_hash, secret_key)
    pub timestamp: u64,          // Unix epoch ms
    pub latency_ms: u64,
}
```

**Verification Flow:**
1. Store receipt in `audit_trail` table
2. On replay, recompute HMAC and compare
3. Mismatch → emit `EXECUTION_INTEGRITY_ERROR` event

### R3: Checkpoint System
SQLite-based checkpoint with WAL mode:

```sql
CREATE TABLE checkpoints (
    id UUID PRIMARY KEY,
    graph_id UUID NOT NULL,
    node_id VARCHAR NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    expires_at TIMESTAMP
);

CREATE INDEX idx_checkpoints_graph ON checkpoints(graph_id);
CREATE INDEX idx_checkpoints_expires ON checkpoints(expires_at);
```

**Checkpoint Strategy:**
- Save after each node completion
- TTL-based expiration (configurable, default 24h)
- Automatic cleanup via cron job
- Crash recovery: reload from latest checkpoint

### R4: Error Handling
| Error Type | Handling |
|------------|----------|
| Node timeout | Retry with backoff (max 3), then fail |
| LLM error | Circuit breaker, fallback model |
| Join timeout | Partial result with warning |
| Human approval pending | Pause graph, resume on signal |

### R5: Testing Requirements
- [ ] Unit tests for each node type
- [ ] Integration test: full graph execution
- [ ] Checkpoint recovery test
- [ ] Receipt verification test
- [ ] Error handling test (timeout, retry, circuit breaker)

## Out of Scope
- Custom node type implementation (future)
- Distributed execution across multiple nodes
- Visual graph editor

## References
- [Phase 2 Research: Rust Graph Execution Engines](../research/rust-graph-execution.md)
- [ri-agent-graph crate](https://crates.io/crates/ri-agent-graph)
- [SPEC-001: EHP Envelope](./SPEC-001-ehp-envelope.md)
