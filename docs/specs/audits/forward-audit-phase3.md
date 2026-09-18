# Forward Audit: Phase 3 Implementation Plan

**Date**: 2026-09-16  
**Auditor**: Lucien (Lead Digital Architect)  
**Scope**: KICKOFF-PHASE3.md, SPEC-001 through SPEC-008, ADR-001 through ADR-008

---

## Audit Methodology

Forward audit validates that the implementation plan correctly implements all specifications and ADRs. Each requirement is traced to plan tasks.

---

## Specification Traceability

### SPEC-001: EHP Envelope
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: Schema definition with fields | 3.1.2 | ✅ Covered |
| R2: Code generation (Rust/Python/TS) | 3.1.2 | ✅ Covered |
| R3: Backward compatibility | 3.1.2 | ⚠️ Needs explicit versioning |
| R4: Validation at message boundaries | 3.2.2 | ✅ Covered |
| R5: Sensitivity labeling | 3.1.2 | ✅ Covered |
| R6: Encryption scope | 3.1.2 | ⚠️ Encryption handling in envelope or storage? |
| R7: TTL for cleanup | 3.2.2 | ✅ Covered via JetStream retention |

**Findings:**
- R3 backward compatibility needs explicit version field in envelope
- R6 encryption scope unclear — envelope should specify requirements, storage handles implementation

### SPEC-002: Warden Blocking
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: Blocking Warden service | 3.4.4 (auth middleware) | ⚠️ Missing dedicated Warden crate |
| R2: NATS request/reply | 3.2.1 | ✅ Covered |
| R3: Fail-closed behavior | 3.4.4 | ✅ Covered |
| R4: Circuit breaker | 3.4.4 | ⚠️ Not explicitly planned |

**Findings:**
- Warden is mentioned in ADR-002 but no dedicated implementation task
- Circuit breaker pattern missing from plan

### SPEC-003: POL Interventions
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: POL Control Plane daemon | 3.4.2 (gRPC service) | ✅ Covered |
| R2: Intervention CRUD | 3.4.2 | ✅ Covered |
| R3: Policy evaluation | 3.6.3 | ✅ Covered |
| R4: Local intervention journal | 3.3.3 (checkpoint) | ⚠️ Different purpose — needs dedicated journal |

**Findings:**
- POL journal is distinct from checkpoint system
- May need separate storage component

### SPEC-004: Graph Execution
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: ri-agent-graph integration | 3.3.1, 3.3.2 | ✅ Covered |
| R2: Cryptographic receipts | 3.3.4 | ✅ Covered |
| R3: Checkpoint system | 3.3.3 | ✅ Covered |
| R4: Error handling | 3.3.2 | ✅ Covered |
| R5: Testing requirements | 3.3.5 | ✅ Covered |

**Findings:**
- All requirements covered
- Receipt generation needs HMAC-SHA256 implementation detail

### SPEC-005: gRPC + WebSocket Daemon
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: gRPC service definition | 3.4.1, 3.4.2 | ✅ Covered |
| R2: WebSocket event streams | 3.4.3 | ✅ Covered |
| R3: Authentication | 3.4.4 | ✅ Covered |
| R4: Connection management | 3.4.3 | ✅ Covered |
| R5: Testing requirements | 3.4.5 | ✅ Covered |

**Findings:**
- All requirements covered
- JWT token format needs specification

### SPEC-006: OpenTelemetry
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: Tracer setup | 3.5.1 | ✅ Covered |
| R2: Span convention | 3.5.1 | ✅ Covered |
| R3: Metrics | 3.5.2 | ✅ Covered |
| R4: Log convention | 3.5.3 | ✅ Covered |
| R5: Exporter config | 3.5.4 | ✅ Covered |
| R6: Testing | 3.5.5 | ✅ Covered |

**Findings:**
- All requirements covered
- Sampling rate (10%) should be configurable

### SPEC-007: OPA Policy Engine
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: Policy structure | 3.6.4 | ✅ Covered |
| R2: Bundle format | 3.6.2 | ✅ Covered |
| R3: Runtime integration | 3.6.1, 3.6.3 | ✅ Covered |
| R4: Evaluation points | 3.6.3 | ✅ Covered |
| R5: Testing | 3.6.5 | ✅ Covered |

**Findings:**
- All requirements covered
- Cache TTL (60s) should be configurable

### SPEC-008: Sandboxing
| Requirement | Plan Task | Status |
|-------------|-----------|--------|
| R1: Mode selection | 3.7.1 | ✅ Covered |
| R2: Security controls | 3.7.2, 3.7.3 | ✅ Covered |
| R3: Resource limits | 3.7.4 | ✅ Covered |
| R4: Execution flow | 3.3.2 (engine) | ⚠️ Flow spans multiple crates |
| R5: Testing | 3.7.5 | ✅ Covered |

**Findings:**
- Execution flow coordination not explicit in plan
- Need orchestration layer between engine and sandbox

---

## ADR Traceability

### ADR-001: NATS JetStream
- Decision: 3-node cluster, R3 replication
- Plan: 3.2.1 uses NATS client
- **Gap**: No infrastructure setup tasks for NATS cluster

### ADR-002: Blocking Warden
- Decision: NATS request/reply, fail-closed
- Plan: 3.4.4 has auth middleware
- **Gap**: Warden service implementation missing

### ADR-003: POL Daemon
- Decision: Daemon on infra VM
- Plan: 3.4.2 implements daemon
- **Gap**: Deployment topology not in plan

### ADR-004: Data Storage
- Decision: PostgreSQL + NATS + Filesystem + Vault
- Plan: 3.3.3 uses SQLite for checkpoints
- **Gap**: PostgreSQL integration missing from plan

### ADR-005: Graph Execution
- Decision: ri-agent-graph v0.2+
- Plan: 3.3.1 adds dependency
- **Gap**: Version pinning not explicit

### ADR-006: Sandboxing
- Decision: nucleus-container strict-agent mode
- Plan: 3.7.1 implements sandbox
- **Gap**: Nix dependency not mentioned

### ADR-007: Observability
- Decision: OpenTelemetry with OTLP
- Plan: 3.5.x covers all requirements
- **Status**: ✅ Fully covered

### ADR-008: Policy Engine
- Decision: OPA/Rego embedded
- Plan: 3.6.x covers all requirements
- **Status**: ✅ Fully covered

---

## Critical Gaps Identified

| Gap | Severity | Recommendation |
|-----|----------|----------------|
| Warden service implementation missing | 🔴 High | Add Phase 3.9: Warden Service |
| PostgreSQL integration missing | 🔴 High | Add Phase 3.10: Database Layer |
| NATS cluster setup missing | 🟡 Medium | Add infrastructure setup documentation |
| POL intervention journal separate from checkpoints | 🟡 Medium | Clarify in SPEC-003 or add dedicated module |
| Sandbox-engine coordination | 🟡 Medium | Add orchestration layer in engine |
| Configuration management | 🟡 Medium | Add config crate for runtime settings |

---

## Recommendations

1. **Add Warden Service Phase**: Dedicated crate for security policy enforcement
2. **Add Database Layer Phase**: PostgreSQL integration for persistent storage
3. **Add Infrastructure Setup Docs**: NATS cluster, PostgreSQL, Vault deployment
4. **Clarify POL Journal**: Either integrate with checkpoint or create separate module
5. **Add Config Management**: Centralized configuration for all runtime settings
6. **Add Deployment Phase**: Container images, systemd services, ansible playbooks

---

## Audit Conclusion

**Status**: ⚠️ REQUIRES REVISION

The implementation plan covers core functionality but has critical gaps in:
- Warden service implementation
- PostgreSQL persistent storage
- Infrastructure deployment

**Recommended action**: Revise KICKOFF.md to include missing phases before sign-off.

---

*Audit completed: 2026-09-16*  
*Auditor: Lucien*
