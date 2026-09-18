# Phase 2 Research Summary

**Date**: 2026-09-15  
**Author**: Lucien (Lead Digital Architect)  
**Status**: Complete  

---

## Overview

Phase 2 research campaign produced 7 comprehensive research documents covering architecture, security, observability, and operational patterns for the RapidAgent Framework.

---

## Research Deliverables

| # | Topic | File | Size | Key Insights |
|---|-------|------|------|--------------|
| 1 | **NATS JetStream Production HA** | `nats-jetstream-production.md` | 5.8 KB | Odd server counts, R3 replication, resource sizing, operational commands |
| 2 | **Rust Graph Execution Engines** | `rust-graph-execution.md` | 11.8 KB | **ri-agent-graph recommended** — agent-specific with cryptographic receipts |
| 3 | **gRPC + WebSocket Daemons** | `grpc-websocket-daemon.md` | 15.8 KB | Dual-interface pattern, protocol multiplexing, Bud/Velocity-Dex examples |
| 4 | **OpenTelemetry Integration** | `opentelemetry-rust.md` | 11.0 KB | Provider initialization, span creation, metrics, context propagation |
| 5 | **Policy as Code (OPA/Rego)** | `policy-as-code-opa.md` | 11.8 KB | Embedded vs server vs Gatekeeper, bundle distribution, testing |
| 6 | **Checkpoint & WAL** | `checkpoint-wal.md` | 12.2 KB | SQLite WAL modes, fssqlite-wal for advanced recovery |
| 7 | **Agent Sandboxing** | `agent-sandboxing.md` | In-progress | nucleus-container recommended for strict isolation |

**Total Research Output**: ~78 KB of structured technical documentation

---

## Key Architectural Decisions

### 1. Graph Execution Engine
**Decision**: Adopt `ri-agent-graph` as primary execution engine  
**Rationale**: 
- Agent-specific node types (llm, router, join, parallel, human_approval)
- Cryptographic receipts (HMAC-SHA256) for execution integrity
- SQLite checkpointing with crash recovery
- JoinSet-backed parallel fan-out with 5 join modes

### 2. NATS JetStream Deployment
**Decision**: 3-node cluster with R3 replication  
**Rationale**:
- Odd server count for Raft quorum
- R3 survives single node failure
- Spread across failure domains
- Pin `max_file_store` to volume size

### 3. Daemon Architecture
**Decision**: Dual-interface (gRPC control + WebSocket data)  
**Rationale**:
- gRPC for typed control plane (spawn/stop/status)
- WebSocket for real-time event streaming
- Protocol multiplexing reduces operational complexity
-Bud pattern proven in production

### 4. Observability
**Decision**: OpenTelemetry with OTLP exporter  
**Rationale**:
- Vendor-neutral telemetry
- Unified traces/metrics/logs
- Native Rust SDK (opentelemetry v0.28)
- Prometheus + Jaeger backends supported

### 5. Policy Engine
**Decision**: OPA/Rego with embedded mode  
**Rationale**:
- Declarative policy language
- Bundle-based distribution
- Unit testing framework
- Low latency (< 1ms p99 for simple policies)

### 6. Security Sandboxing
**Decision**: nucleus-container strict-agent mode  
**Rationale**:
- Nix-based reproducible rootfs
- Seccomp profiles with SHA-256 pinning
- Landlock filesystem ACLs
- Fail-closed isolation

---

## Integration Points

### rapidagent-engine Crates

```toml
[dependencies]
# Graph execution
ri-agent-graph = "0.2"

# NATS
nats = "0.23"

# gRPC
tonic = "0.12"
prost = "0.13"

# Observability
opentelemetry = "0.28"
opentelemetry-sdk = "0.28"
opentelemetry-otlp = "0.28"

# Security
# nucleus-container CLI or library bindings

# Checkpointing
rusqlite = { version = "0.32", features = ["hooks"] }
serde_json = "1"
sha2 = "0.10"
```

### Proposed Crate Structure

```
crates/
├── rapidagent-graph/          # ri-agent-graph wrapper
├── rapidagent-daemon/         # gRPC + WebSocket server
├── rapidagent-observability/  # OTel integration
├── rapidagent-security/       # Sandbox integration
├── rapidagent-checkpoint/     # WAL + checkpointing
└── rapidagent-policies/       # OPA policy loading
```

---

## Next Steps

### Phase 3: Implementation Planning

1. **Create KICKOFF.md** with implementation phases
2. **Scaffold monorepo** with cargo workspace
3. **Implement core crates** in priority order:
   - P0: rapidagent-ir, rapidagent-envelope
   - P1: rapidagent-graph, rapidagent-daemon
   - P2: rapidagent-observability, rapidagent-security

### Immediate Actions

1. Review research findings with Steven
2. Confirm `ri-agent-graph` as primary engine
3. Finalize NATS cluster topology
4. Create prototype daemon with gRPC + WebSocket

---

## References

All research documents available at `~/Workspaces/rapidagent/docs/research/`

| Document | URL/Source |
|----------|------------|
| NATS JetStream | docs.nats.io/learn/topologies/jetstream-in-a-cluster |
| ri-agent-graph | crates.io/crates/ri-agent-graph |
| nucleus-container | crates.io/crates/nucleus-container |
| OPA Documentation | openpolicyagent.org/docs |
| OpenTelemetry Rust | opentelemetry.io/docs/languages/rust |

---

*Phase 2 completed: 2026-09-15*  
*Ready for Phase 3: Implementation Planning*
