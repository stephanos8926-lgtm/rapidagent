# ADR-005: Graph Execution Engine Selection

---
title: "ADR-005: Graph Execution Engine Selection"
status: Accepted
date: 2026-09-16
deciders: Lucien, Steven Page
consulted: RapidWebs Engineering Team
informed: All contributors

---

## Context

The RapidAgent Framework requires an execution engine capable of:
- DAG-based workflow execution
- Parallel fan-out with configurable join strategies
- Cryptographic execution receipts for auditability
- Crash recovery via checkpointing
- Integration with NATS JetStream events

Options evaluated: custom implementation, `ri-agent-graph`, `daggy`, `petgraph` + custom.

## Decision

We will use **ri-agent-graph** v0.2+ as the primary execution engine.

## Alternatives Considered

| Option | Pros | Cons | Reason Rejected |
|--------|------|------|-----------------|
| **Custom implementation** | Full control | High development cost, reinventing wheel | `ri-agent-graph` is production-ready |
| **daggy** | Simple API, well-tested | No cryptographic receipts, no checkpointing | Lacks security features required |
| **petgraph + custom** | Maximum flexibility | High complexity, longer development | Over-engineered for our needs |
| **ri-agent-graph** ✅ | Agent-specific types, crypto receipts, checkpointing | Newer crate, smaller community | Meets all requirements |

## Consequences

### Positive
- **Cryptographic receipts**: HMAC-SHA256 for execution integrity
- **Crash recovery**: SQLite checkpointing with WAL
- **Agent-specific nodes**: llm, router, join, parallel, human_approval
- **Mature integration**: Proven in production agent systems
- **Low overhead**: <5% performance impact

### Negative
- **Newer crate**: Smaller community, less documentation
- **Version lock**: Tied to ri-agent-graph release cycle
- **Learning curve**: Team needs training on API

### Mitigations
- Monitor crates.io for updates
- Contribute back to upstream
- Document internal patterns in AGENTS.md

## Implementation Notes

- Add `ri-agent-graph = "0.2"` to `crates/engine/Cargo.toml`
- Implement checkpoint manager using SQLite
- Integrate with EHP envelope system (SPEC-001)
- Add cryptographic receipt generation in execution loop

## References
- [SPEC-004: Graph Execution Engine](../specs/SPEC-004-graph-execution.md)
- [ri-agent-graph crate](https://crates.io/crates/ri-agent-graph)
