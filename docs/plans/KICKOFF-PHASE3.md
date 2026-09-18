# KICKOFF: RapidAgent Framework Phase 3 Implementation

**Date**: 2026-09-16  
**Author**: Lucien  
**Status**: Draft — Pending Sign-off  
**Mode**: MEDIUM (multi-phase, 8+ files, database/integration dependencies)

---

## Executive Summary

This document initiates Phase 3 of the RapidAgent Framework implementation. Phase 2 research established architecture decisions (SPEC-001 through SPEC-008, ADR-001 through ADR-008). Phase 3 translates these into executable implementation tasks.

**Current State:**
- ✅ Phase 1: Architecture foundation (component list, audits, data storage design)
- ✅ Phase 2: Research campaign (7 technical research documents)
- ✅ SPECs created: EHP Envelope, Warden Blocking, POL Interventions, Graph Execution, gRPC Daemon, OpenTelemetry, OPA Policy, Sandboxing
- ✅ ADRs created: NATS JetStream, Blocking Warden, POL Daemon, Data Storage, Graph Engine, Sandboxing, Observability, Policy Engine
- ✅ Python crate dependency fixed (30 tests passing)
- ⏭️ Phase 3: Implementation planning and execution

---

## Implementation Phases

### Phase 3.1: Foundation Crates (P0)
**Goal**: Establish core type system and message schema

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.1.1 | `rapidagent-ir` extensions | ir/src/types.rs, ir/src/lib.rs | 2h |
| 3.1.2 | `rapidagent-envelope` crate | crates/envelope/src/*.rs, schemas/ehp-envelope-v1.json | 4h |
| 3.1.3 | Type-safe NATS subjects | crates/subjects/src/*.rs | 2h |
| 3.1.4 | Cross-crate integration tests | tests/integration/*.rs | 2h |

**Deliverables:**
- Compiled IR types with graph structures
- EHP Envelope v1 schema in Rust, Python, TypeScript
- Type-safe subject builders (no string typos)
- Integration tests passing

---

### Phase 3.2: Event System (P0)
**Goal**: Implement NATS-backed event emitter/subscriber

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.2.1 | NATS client wrapper | crates/events/src/nats_client.rs | 3h |
| 3.2.2 | Event emitter API | crates/events/src/emitter.rs | 2h |
| 3.2.3 | Event subscriber API | crates/events/src/subscriber.rs | 2h |
| 3.2.4 | In-process fallback (testing) | crates/events/src/inprocess.rs | 1h |
| 3.2.5 | Integration tests | tests/test_events.rs | 2h |

**Deliverables:**
- NATS JetStream integration
- Publish/subscribe with EHP envelope validation
- Graceful fallback for offline mode

---

### Phase 3.3: Graph Execution Engine (P0)
**Goal**: Integrate ri-agent-graph with cryptographic receipts

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.3.1 | Add ri-agent-graph dependency | Cargo.toml | 0.5h |
| 3.3.2 | Graph execution wrapper | crates/engine/src/graph_executor.rs | 4h |
| 3.3.3 | Checkpoint manager | crates/engine/src/checkpoint.rs | 3h |
| 3.3.4 | Receipt generation | crates/engine/src/receipt.rs | 2h |
| 3.3.5 | Unit tests | tests/test_graph_execution.rs | 3h |

**Deliverables:**
- DAG execution with parallel fan-out
- HMAC-SHA256 cryptographic receipts
- SQLite checkpointing with crash recovery
- 80%+ test coverage

---

### Phase 3.4: gRPC + WebSocket Daemon (P1)
**Goal**: Dual-interface daemon for agent control and streaming

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.4.1 | Protobuf definitions | protos/agent_control.proto | 1h |
| 3.4.2 | gRPC service implementation | crates/daemon/src/grpc_service.rs | 4h |
| 3.4.3 | WebSocket handler | crates/daemon/src/websocket.rs | 3h |
| 3.4.4 | Authentication middleware | crates/daemon/src/auth.rs | 2h |
| 3.4.5 | Integration tests | tests/test_daemon.rs | 3h |

**Deliverables:**
- gRPC service: Spawn, Stop, Status, List, Pause, History
- WebSocket streams: logs, metrics, state, receipts, errors
- mTLS + JWT authentication
- 100 concurrent agent support

---

### Phase 3.5: Observability (P1)
**Goal**: OpenTelemetry integration for traces, metrics, logs

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.5.1 | OTel tracer setup | crates/observability/src/tracer.rs | 2h |
| 3.5.2 | Metric definitions | crates/observability/src/metrics.rs | 2h |
| 3.5.3 | Log integration | crates/observability/src/logs.rs | 1h |
| 3.5.4 | OTLP exporter config | crates/observability/src/exporter.rs | 1h |
| 3.5.5 | Tests | tests/test_observability.rs | 2h |

**Deliverables:**
- Tracer with parent-based sampling
- Standardized metrics (agent.active, node.duration, etc.)
- Structured logging with OTel subscriber
- Prometheus + Jaeger backend support

---

### Phase 3.6: Policy Engine (P1)
**Goal**: Embedded OPA/Rego for policy evaluation

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.6.1 | OPA runtime integration | crates/policies/src/runtime.rs | 3h |
| 3.6.2 | Bundle loader | crates/policies/src/bundle.rs | 2h |
| 3.6.3 | Policy evaluation API | crates/policies/src/evaluator.rs | 2h |
| 3.6.4 | Sample policies | docs/policies/*.rego | 1h |
| 3.6.5 | Tests | tests/test_policies.rs | 2h |

**Deliverables:**
- Embedded OPA runtime (<1ms p99)
- Bundle-based policy distribution
- Cache with TTL invalidation
- Unit tests for each policy package

---

### Phase 3.7: Sandboxing (P2)
**Goal**: nucleus-container integration for agent isolation

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.7.1 | Sandbox builder | crates/sandbox/src/builder.rs | 3h |
| 3.7.2 | Seccomp profile loader | crates/sandbox/src/seccomp.rs | 2h |
| 3.7.3 | Landlock rules | crates/sandbox/src/landlock.rs | 2h |
| 3.7.4 | Resource limits | crates/sandbox/src/resources.rs | 1h |
| 3.7.5 | Tests | tests/test_sandbox.rs | 3h |

**Deliverables:**
- strict-agent mode sandboxing
- SHA-256 pinned seccomp profiles
- Landlock filesystem ACLs
- Resource limits enforcement

---

### Phase 3.8: Integration & E2E (P2)
**Goal**: End-to-end validation of all components

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.8.1 | Hello daemon scenario | tests/e2e/test_hello_daemon.rs | 3h |
| 3.8.2 | Multi-agent stress test | tests/e2e/test_stress.rs | 2h |
| 3.8.3 | Policy intervention test | tests/e2e/test_policy.rs | 2h |
| 3.8.4 | Checkpoint recovery test | tests/e2e/test_recovery.rs | 2h |
| 3.8.5 | Documentation update | AGENTS.md, README.md | 1h |

**Deliverables:**
- Full workflow validation
- Performance benchmarks
- Updated documentation

---

## Total Effort Estimate

| Phase | Est. Hours | Priority |
|-------|-----------|----------|
| 3.1 Foundation Crates | 10h | P0 |
| 3.2 Event System | 10h | P0 |
| 3.3 Graph Execution | 14h | P0 |
| 3.4 gRPC + WebSocket Daemon | 13h | P1 |
| 3.5 Observability | 8h | P1 |
| 3.6 Policy Engine | 10h | P1 |
| 3.7 Sandboxing | 11h | P2 |
| 3.8 Integration & E2E | 10h | P2 |
| **TOTAL** | **86h** | — |

---

## Dependencies

| Phase | Depends On |
|-------|------------|
| 3.2 | 3.1 (envelope, subjects) |
| 3.3 | 3.1 (IR types) |
| 3.4 | 3.2 (events), 3.3 (engine) |
| 3.5 | 3.2 (events) |
| 3.6 | 3.4 (daemon) |
| 3.7 | 3.3 (engine) |
| 3.8 | All previous phases |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ri-agent-graph API changes | Medium | Medium | Pin version, monitor upstream |
| NATS connection issues | Low | High | In-process fallback, retry logic |
| OPA bundle loading failure | Low | Medium | Graceful degradation, cache fallback |
| Sandbox creation timeout | Medium | Medium | Timeout with clear error message |
| Test coverage <80% | Medium | Low | TDD enforced, incremental coverage |

---

## Success Criteria

- [ ] All 30 existing tests pass
- [ ] New tests: 80%+ coverage on P0 components
- [ ] gRPC service responds <10ms p99
- [ ] Policy evaluation <1ms p99
- [ ] Checkpoint recovery <5s
- [ ] E2E hello-daemon scenario passes
- [ ] Documentation updated

---

## Next Steps

1. **Review** this KICKOFF.md
2. **Sign-off** on implementation phases
3. **Begin** Phase 3.1 (Foundation Crates)

---

*Document version: 1.0*  
*Last updated: 2026-09-16*
