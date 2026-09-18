# Synthesis: Phase 3 Implementation Plan v2

**Date**: 2026-09-16  
**Author**: Lucien  
**Based on**: KICKOFF-PHASE3.md + Forward Audit + Reverse Audit

---

## Executive Summary

Phase 3 implementation plan has been revised based on forward and reverse audits. Key changes:
- Added missing critical components (Warden, Database, Config)
- Simplified over-engineered areas (sandbox integration, observability inline)
- Deferred non-critical features (WebSocket P2, TS SDK P4)
- Added testing infrastructure and CLI

**Total revised effort**: 96 hours (was 86h, +10h for missing components)

---

## Revised Implementation Phases

### Phase 3.0: Foundation & Tooling (NEW)
**Goal**: Establish cross-cutting concerns before domain crates

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.0.1 | Error handling crate | crates/errors/src/*.rs | 2h |
| 3.0.2 | Config management crate | crates/config/src/*.rs | 3h |
| 3.0.3 | Logging setup | crates/logging/src/*.rs | 2h |
| 3.0.4 | Test helpers crate | crates/test-helpers/src/*.rs | 3h |
| 3.0.5 | Workspace Cargo.toml cleanup | Cargo.toml | 1h |

**Deliverables:**
- Standardized error types with context chaining
- Centralized configuration with hot-reload
- Structured JSON logging with rotation
- Mock NATS, test PostgreSQL, time mocking

---

### Phase 3.1: Foundation Crates (P0)
**Goal**: Establish core type system and message schema

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.1.1 | `rapidagent-ir` extensions | ir/src/types.rs | 2h |
| 3.1.2 | `rapidagent-envelope` crate | crates/envelope/src/*.rs | 4h |
| 3.1.3 | Type-safe NATS subjects | crates/subjects/src/*.rs | 2h |
| 3.1.4 | Integration tests | tests/integration/*.rs | 2h |

**Changes from v1:**
- Add `correlation_id` field to envelope (audit finding)
- Add `schema_version` field (audit finding)
- In-line observability (no separate crate)

---

### Phase 3.2: Event System (P0)
**Goal**: Implement NATS-backed event emitter/subscriber

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.2.1 | NATS client wrapper | crates/events/src/nats_client.rs | 3h |
| 3.2.2 | Event emitter API | crates/events/src/emitter.rs | 2h |
| 3.2.3 | Event subscriber API | crates/events/src/subscriber.rs | 2h |
| 3.2.4 | In-process fallback | crates/events/src/inprocess.rs | 1h |
| 3.2.5 | Tests | tests/test_events.rs | 2h |

---

### Phase 3.3: Graph Execution Engine (P0)
**Goal**: Integrate ri-agent-graph with cryptographic receipts

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.3.1 | Add ri-agent-graph dependency | Cargo.toml | 0.5h |
| 3.3.2 | Graph execution wrapper | crates/engine/src/graph_executor.rs | 4h |
| 3.3.3 | Checkpoint manager | crates/engine/src/checkpoint.rs | 3h |
| 3.3.4 | Receipt generation | crates/engine/src/receipt.rs | 2h |
| 3.3.5 | Sandbox integration | crates/engine/src/sandbox.rs | 3h |
| 3.3.6 | Tests | tests/test_graph_execution.rs | 3h |

**Changes from v1:**
- Integrate sandbox into engine (not separate crate)
- Pin ri-agent-graph version: `=0.2.3`

---

### Phase 3.4: Database Layer (NEW)
**Goal**: PostgreSQL integration for persistent storage

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.4.1 | PostgreSQL connection pool | crates/database/src/pool.rs | 3h |
| 3.4.2 | Schema migrations | migrations/*.sql | 2h |
| 3.4.3 | Agent registry | crates/database/src/agents.rs | 3h |
| 3.4.4 | Audit trail | crates/database/src/audit.rs | 2h |
| 3.4.5 | POL interventions | crates/database/src/pol.rs | 2h |
| 3.4.6 | Tests | tests/test_database.rs | 3h |

**Schema:**
```sql
CREATE TABLE agents (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    template_hash VARCHAR,
    status VARCHAR,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE audit_trail (
    id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(id),
    event_type VARCHAR NOT NULL,
    payload JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE pol_interventions (
    id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(id),
    intervention_type VARCHAR,
    status VARCHAR,
    created_at TIMESTAMP DEFAULT NOW()
);
```

---

### Phase 3.5: Warden Service (NEW)
**Goal**: Security policy enforcement service

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.5.1 | Warden service definition | crates/warden/src/service.rs | 3h |
| 3.5.2 | Policy evaluation | crates/warden/src/policy.rs | 3h |
| 3.5.3 | Circuit breaker | crates/warden/src/circuit_breaker.rs | 2h |
| 3.5.4 | NATS request/reply handler | crates/warden/src/nats_handler.rs | 2h |
| 3.5.5 | Tests | tests/test_warden.rs | 3h |

**Key decisions:**
- Fail-closed: timeout or error → deny
- Circuit breaker: 5 failures in 60s → open for 30s
- Policy evaluation cached (TTL 60s)

---

### Phase 3.6: gRPC Daemon (P1)
**Goal**: Dual-interface daemon (gRPC control plane)

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.6.1 | Protobuf definitions | protos/agent_control.proto | 1h |
| 3.6.2 | gRPC service implementation | crates/daemon/src/grpc_service.rs | 4h |
| 3.6.3 | Authentication middleware | crates/daemon/src/auth.rs | 2h |
| 3.6.4 | mTLS setup | crates/daemon/src/tls.rs | 2h |
| 3.6.5 | Tests | tests/test_daemon.rs | 3h |

**Deferred to P2:**
- WebSocket streaming (spec incomplete, gRPC streaming sufficient initially)

---

### Phase 3.7: Policy Engine (P1)
**Goal**: Embedded OPA/Rego for policy evaluation

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.7.1 | OPA runtime integration | crates/policies/src/runtime.rs | 3h |
| 3.7.2 | Simple policy loader | crates/policies/src/loader.rs | 2h |
| 3.7.3 | Policy evaluation API | crates/policies/src/evaluator.rs | 2h |
| 3.7.4 | Sample policies | docs/policies/*.rego | 1h |
| 3.7.5 | Tests | tests/test_policies.rs | 2h |

**Simplification from v1:**
- Skip bundle format complexity, use direct Rego file loading
- Add bundle support in Phase 4 if needed

---

### Phase 3.8: CLI Tool (NEW)
**Goal**: Developer-facing command-line interface

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.8.1 | CLI entry point | crates/cli/src/main.rs | 1h |
| 3.8.2 | Compile command | crates/cli/src/compile.rs | 2h |
| 3.8.3 | Run command | crates/cli/src/run.rs | 2h |
| 3.8.4 | Inspect command | crates/cli/src/inspect.rs | 2h |
| 3.8.5 | Tests | tests/test_cli.rs | 2h |

---

### Phase 3.9: Integration & E2E (P2)
**Goal**: End-to-end validation

| Task | Component | Files | Est. |
|------|-----------|-------|------|
| 3.9.1 | Hello daemon scenario | tests/e2e/test_hello_daemon.rs | 3h |
| 3.9.2 | Multi-agent stress test | tests/e2e/test_stress.rs | 2h |
| 3.9.3 | Policy intervention test | tests/e2e/test_policy.rs | 2h |
| 3.9.4 | Checkpoint recovery test | tests/e2e/test_recovery.rs | 2h |
| 3.9.5 | Documentation | AGENTS.md, README.md | 1h |

---

## Revised Effort Estimate

| Phase | Est. Hours | Priority |
|-------|-----------|----------|
| 3.0 Foundation & Tooling | 11h | P0 |
| 3.1 Foundation Crates | 10h | P0 |
| 3.2 Event System | 10h | P0 |
| 3.3 Graph Execution | 17h | P0 |
| 3.4 Database Layer | 15h | P0 |
| 3.5 Warden Service | 13h | P0 |
| 3.6 gRPC Daemon | 12h | P1 |
| 3.7 Policy Engine | 10h | P1 |
| 3.8 CLI Tool | 9h | P1 |
| 3.9 Integration & E2E | 10h | P2 |
| **TOTAL** | **117h** | — |

**Comparison:**
- Original plan: 86h
- Revised plan: 117h (+31h)
- Reason: Added critical missing components (Warden, Database, Config, CLI, Errors)

---

## Dependency Graph

```
3.0 Foundation → 3.1 IR/Envelope → 3.2 Events
                      ↓
3.3 Graph Engine ← 3.4 Database
      ↓              ↓
3.5 Warden ← 3.6 gRPC Daemon
      ↓              ↓
3.7 Policy ← 3.8 CLI
      ↓
3.9 E2E Tests
```

---

## Key Decisions from Audit

### Decisions Made
1. ✅ Integrate sandbox into engine (not separate crate)
2. ✅ Inline observability (not separate crate)
3. ✅ Defer WebSocket to P2
4. ✅ Defer TypeScript SDK to P4
5. ✅ Simplify policy loading (skip bundles initially)
6. ✅ Add Warden as separate crate (security boundary)
7. ✅ Add Database layer (persistent storage requirement)
8. ✅ Add Config management (cross-cutting concern)
9. ✅ Add Error handling crate (consistency)
10. ✅ Add CLI tool (developer experience)

### Open Questions
1. Should POL intervention journal be separate from checkpoint system?
2. What JWT signing algorithm for daemon auth?
3. Should we use managed NATS or self-hosted for Horizon 1?
4. What's the rollback strategy if Phase 3 fails?

---

## Success Criteria (Revised)

- [ ] All 30 existing tests pass
- [ ] New tests: 80%+ coverage on P0 components
- [ ] gRPC service responds <10ms p99
- [ ] Policy evaluation <1ms p99
- [ ] Checkpoint recovery <5s
- [ ] Warden fail-closed on timeout
- [ ] Circuit breaker trips after 5 failures
- [ ] E2E hello-daemon scenario passes
- [ ] Documentation updated

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| ri-agent-graph API changes | Pin version =0.2.3, fork if needed |
| NATS connection issues | In-process fallback in tests |
| PostgreSQL migration failures | Idempotent migrations, rollback scripts |
| OPA learning curve | Documentation, training session |
| Scope creep | Strict phase boundaries, defer to P4 |

---

## Next Steps

1. **Review** this synthesis document
2. **Sign-off** on revised implementation plan
3. **Begin** Phase 3.0 (Foundation & Tooling)

---

*Document version: 2.0*  
*Last updated: 2026-09-16*  
*Based on: KICKOFF-PHASE3.md + forward/reverse audits*
