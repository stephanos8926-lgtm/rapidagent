# AGENTS.md — RapidAgent Framework

> **Project Context and Operating Procedures for AI Agents**
> 
> Last updated: 2026-09-16
> Version: 1.2
> Maintainer: Lucien (Lead Digital Architect)

---

## 1. Project Overview

**Project**: RapidAgent Framework  
**Status**: Phase 3 — Implementation Planning (Pending Sign-off)  
**Repository**: `~/Workspaces/rapidagent/`  
**Branch**: `main` (work in progress)

### Mission Statement
Build an enterprise-grade agent orchestration platform with automated governance, security enforcement, and multi-tenant support.

### Key Technologies
- **Rust** — Core framework, compiler, engine, daemon
- **Python** — SDK, tools, POL Agent integration
- **TypeScript** — IDE extensions, tooling
- **NATS JetStream** — Event transport
- **PostgreSQL 16** — Persistent storage, pgvector
- **Hashicorp Vault** — Secrets management
- **gRPC/Protobuf** — Service communication
- **OpenTelemetry** — Observability
- **OPA/Rego** — Policy engine
- **nucleus-container** — Sandboxing

---

## 2. Repository Structure

```
rapidagent/
├── crates/                    # Rust workspace
│   ├── core/                  # Lexer, parser, optimizer
│   ├── ir/                    # Intermediate representation types
│   ├── renderer/              # Output formatters (xml, md, json, folder, binary)
│   ├── security/              # Injection scanning, hashing, signing
│   ├── python/                # PyO3 bindings
│   ├── versioning/            # Semver, UUIDv5 provenance
│   ├── ml-eval/               # ML evaluation helpers
│   ├── cli/                   # CLI entry point
│   ├── envelope/              # EHP envelope schema (P3)
│   ├── subjects/              # Type-safe NATS subjects (P3)
│   ├── events/                # Event system (P3)
│   ├── engine/                # Graph execution engine (P3)
│   ├── database/              # PostgreSQL layer (P3)
│   ├── warden/                # Security Warden (P3)
│   ├── daemon/                # gRPC daemon (P3)
│   ├── policies/              # OPA policy engine (P3)
│   ├── errors/                # Error handling (P3)
│   ├── config/                # Configuration management (P3)
│   ├── logging/               # Logging setup (P3)
│   └── test-helpers/          # Test utilities (P3)
├── docs/
│   ├── plans/                 # Architecture plans
│   ├── specs/                 # Technical specifications
│   │   └── audits/            # Audit reports
│   ├── adrs/                  # Architecture decision records
│   ├── research/              # Research findings
│   ├── arxiv/                 # Academic papers
│   └── bookmarks/             # Reference links
├── protos/                    # Protobuf definitions
├── migrations/                # Database migrations
├── tests/                     # Test suite
├── schemas/                   # JSON Schema definitions
├── config/                    # Configuration templates
├── scripts/                   # Build/deploy scripts
├── ansible/                   # Infrastructure deployment
└── .github/                   # CI/CD workflows
```

---

## 3. Key Decisions (ADR Summary)

| ADR | Topic | Decision |
|-----|-------|----------|
| **ADR-001** | Event Transport | NATS JetStream for all inter-component events |
| **ADR-002** | Security Gate | Blocking Warden via NATS request/reply |
| **ADR-003** | POL Architecture | Daemon service on infra VM (not embedded) |
| **ADR-004** | Data Storage | PostgreSQL + NATS + Filesystem + Vault |
| **ADR-005** | Graph Engine | ri-agent-graph with cryptographic receipts |
| **ADR-006** | Sandboxing | nucleus-container strict-agent mode |
| **ADR-007** | Observability | OpenTelemetry with OTLP exporter |
| **ADR-008** | Policy Engine | OPA/Rego embedded mode |

### Critical Decisions Summary

1. **NATS JetStream** — Primary event transport with HA cluster planned
2. **PostgreSQL 16** — Structured data with pgvector for embeddings
3. **Hashicorp Vault** — Secrets management (API keys, signing keys)
4. **Ed25519 Signing** — Artifact provenance via UUIDv5(SHA256(payload) + pubkey)
5. **Fail-closed Warden** — Timeout or error → deny operation
6. **Multi-tenancy** — Single-tenant for Horizon 1, namespace isolation
7. **ri-agent-graph** — Agent-specific execution with HMAC-SHA256 receipts
8. **nucleus-container** — Nix-based reproducible sandboxing

---

## 4. Current Build Status

| Crate | Tests | Status |
|-------|-------|--------|
| rapidagent-core | 11 | ✅ Pass |
| rapidagent-ir | 6 | ✅ Pass |
| rapidagent-renderer | 8 | ✅ Pass |
| rapidagent-security | 5 | ✅ Pass |
| rapidagent-python | 0 | ✅ Pass (fixed) |
| **Total** | **30** | **✅ All Pass** |

---

## 5. Phase 3 Implementation Plan (Pending Sign-off)

### Phase 3.0: Foundation & Tooling (P0)
- Error handling crate
- Config management crate
- Logging setup
- Test helpers

### Phase 3.1: Foundation Crates (P0)
- `rapidagent-ir` extensions
- `rapidagent-envelope` crate
- Type-safe NATS subjects

### Phase 3.2: Event System (P0)
- NATS client wrapper
- Event emitter/subscriber

### Phase 3.3: Graph Execution Engine (P0)
- ri-agent-graph integration
- Checkpoint manager
- Cryptographic receipts
- Sandbox integration

### Phase 3.4: Database Layer (P0)
- PostgreSQL connection pool
- Schema migrations
- Agent registry, audit trail, POL interventions

### Phase 3.5: Warden Service (P0)
- Security policy enforcement
- Circuit breaker
- Fail-closed behavior

### Phase 3.6: gRPC Daemon (P1)
- Protobuf definitions
- Service implementation
- Authentication (mTLS + JWT)

### Phase 3.7: Policy Engine (P1)
- OPA runtime integration
- Policy evaluation API
- Sample policies

### Phase 3.8: CLI Tool (P1)
- Compile, run, inspect commands

### Phase 3.9: Integration & E2E (P2)
- Hello daemon scenario
- Stress tests
- Policy intervention tests
- Recovery tests

**Total Effort**: 117 hours

---

## 6. Development Workflow

### Building

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build -p rapidagent-core --release

# Run tests
cargo test --workspace

# Run specific test
cargo test -p rapidagent-core test_parse_tokens
```

### Testing

```bash
# Unit tests
cargo test --workspace

# Integration tests (requires NATS + PG)
NATS_URL=nats://localhost:4222 PG_URL=postgres://localhost/rapidagent cargo test --test integration

# Run with coverage
cargo tarpaulin --workspace --out Xml
```

### Code Style

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Python**: PEP 8 + `black` formatter
- **Commits**: Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`)

---

## 7. Architecture Components

### Tier 1: Infrastructure (P0)
- `INF-001`: NATS JetStream Cluster
- `INF-002`: PostgreSQL 16
- `INF-003`: Caddy Reverse Proxy
- `INF-004`: Tailscale Mesh

### Tier 2: Core Framework (P0-P1)
- `FWD-001`: `rapidagent-ir` — IR types
- `FWD-002`: `rapidagent-envelope` — EHP schema
- `FWD-003`: `rapidagent-events` — Event system
- `FWD-009`: `rapidagent-subjects` — NATS subject constants

### Tier 3: Compiler (P0)
- `CMP-001`: `rw-syspro-compiler` — Existing compiler
- `CMP-002`: DAG Validator
- `CMP-003`: Policy Compiler

### Tier 4: Engine (P0)
- `ENG-001`: `rapidagent-engine` — Execution engine
- `ENG-002`: Checkpoint Manager
- `ENG-004`: Policy Hook
- `ENG-010`: Checkpoint WAL

### Tier 5: POL (P0)
- `POL-001`: `POLControlPlane` — Daemon service
- `POL-002`: Policy Evaluator
- `POL-009`: Local Intervention Journal

### Tier 6: Warden (P0)
- `WRD-001`: `SecurityWarden` — RBAC gate
- `WRD-002`: Request Interceptors
- `WRD-006`: Circuit Breaker

---

## 8. Data Storage Strategy

### PostgreSQL Tables
- `agents` — Agent registry
- `interventions` — POL interventions
- `warden_decisions` — Security Warden decisions
- `audit_trail` — Immutable audit log
- `policies` — Policy configurations
- `embeddings` — Vector embeddings (pgvector)

### NATS Streams
- `agents.{tenant}.{id}.events` — Agent lifecycle
- `pol.{tenant}.interventions` — POL events
- `warden.{tenant}.requests` — Warden checks
- `telemetry.{tenant}.events` — Metrics

---

## 9. Documentation

### Specifications
- [SPEC-001: EHP Envelope](docs/specs/SPEC-001-ehp-envelope.md)
- [SPEC-002: Warden Blocking](docs/specs/SPEC-002-warden-blocking.md)
- [SPEC-003: POL Interventions](docs/specs/SPEC-003-pol-interventions.md)
- [SPEC-004: Graph Execution](docs/specs/SPEC-004-graph-execution.md)
- [SPEC-005: gRPC Daemon](docs/specs/SPEC-005-grpc-websocket-daemon.md)
- [SPEC-006: OpenTelemetry](docs/specs/SPEC-006-opentelemetry.md)
- [SPEC-007: OPA Policy](docs/specs/SPEC-007-opa-policy.md)
- [SPEC-008: Sandboxing](docs/specs/SPEC-008-sandboxing.md)

### Architecture Decision Records
- [ADR-001: NATS JetStream](docs/adrs/ADR-001-nats-jetstream.md)
- [ADR-002: Blocking Warden](docs/adrs/ADR-002-blocking-warden.md)
- [ADR-003: POL Daemon](docs/adrs/ADR-003-pol-daemon.md)
- [ADR-004: Data Storage](docs/adrs/ADR-004-data-storage.md)
- [ADR-005: Graph Execution](docs/adrs/ADR-005-graph-execution.md)
- [ADR-006: Sandboxing](docs/adrs/ADR-006-sandboxing.md)
- [ADR-007: Observability](docs/adrs/ADR-007-observability.md)
- [ADR-008: Policy Engine](docs/adrs/ADR-008-policy-engine.md)

### Implementation Plans
- [KICKOFF: Phase 3](docs/plans/KICKOFF-PHASE3.md)
- [Synthesis v2](docs/specs/audits/synthesis-phase3-v2.md)

### Research
- [Phase 2 Summary](docs/research/PHASE2-SUMMARY.md)
- [NATS JetStream](docs/research/nats-jetstream-production.md)
- [Graph Execution](docs/research/rust-graph-execution.md)
- [gRPC + WebSocket](docs/research/grpc-websocket-daemon.md)
- [OpenTelemetry](docs/research/opentelemetry-rust.md)
- [OPA Policy](docs/research/policy-as-code-opa.md)
- [Checkpoint & WAL](docs/research/checkpoint-wal.md)
- [Sandboxing](docs/research/agent-sandboxing.md)

---

## 10. Next Steps

### Immediate (Pending Sign-off)
1. Review KICKOFF-PHASE3.md and synthesis document
2. Approve implementation plan
3. Begin Phase 3.0 (Foundation & Tooling)

### Short-term (Next 2 Weeks)
1. Complete Phase 3.0-3.3 (Foundation + Event System + Graph Engine)
2. Complete Phase 3.4-3.5 (Database + Warden)
3. Integration testing

### Medium-term (Next Month)
1. Complete Phase 3.6-3.8 (Daemon + Policy + CLI)
2. E2E validation
3. Documentation completion

---

## 11. Contact

**Lead Architect**: Lucien (sysop)  
**Email**: sysop@rapidwebs  
**Telegram**: @lucien_rapidwebs

---

*End of AGENTS.md*
