---
title: "RapidAgent Framework — Architecture Component List"
description: "Complete inventory of all components, services, and subsystems required to realize the RapidAgent Framework vision"
status: draft
version: 2.0
created: 2026-09-15
updated: 2026-09-15
author: Lucien (Lead Digital Architect, RapidWebs Enterprise)
related-docs:
  - "RapidAgent Framework Architecture and Design Vision Document.md"
  - "audit-forward.md"
  - "audit-reverse.md"
  - "audit-forward-inline.md"
  - "audit-reverse-inline.md"
  - "audit-adversarial.md"
  - "forward-audit-report.md"
---

# Architecture Component List — RapidAgent Framework

> **Purpose**: Complete inventory of all components required to build the RapidAgent Framework.
> **Scope**: Foundation layer through production deployment.
> **Version**: 2.0 (post-audit synthesis)
> **Audit Status**: ✅ Forward ✅ Reverse ✅ Adversarial ✅ Inline

---

## Component Inventory

### Tier 1: Infrastructure & Foundation

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 1 | `INF-001` | NATS JetStream Cluster | Service | Go (upstream) | Primary event transport; durable streams, KV, object store. Deployed on `infra` VM. | P0 |
| 2 | `INF-002` | PostgreSQL 16 | Database | Rust (connector) | Agent registry, audit logs, Honcho memory, POL interventions. | P0 |
| 3 | `INF-003` | Caddy Reverse Proxy | Service | Go | TLS termination, path-based routing for gRPC/WS/HTTP APIs. | P1 |
| 4 | `INF-004` | Tailscale Mesh | Network | Node (upstream) | Private networking across workstations + VMs. | P1 |
| 5 | `INF-005` | Git Worktree Manager | Tooling | Rust/Python | Isolated dev environments per task/feature branch. | P2 |
| 6 | `INF-006` | Podman Container Runtime | Platform | Linux | Production container orchestration (optional for daemon). | P2 |
| 7 | **`INF-007`** ⭐ | **NATS HA Cluster** | Service | Go (upstream) | **NEW: Multi-node NATS cluster for high availability. Eliminates single point of failure.** | **P0** |
| 8 | **`INF-008`** ⭐ | **PG Replica** | Database | Rust (connector) | **NEW: Streaming replication for PostgreSQL. Provides HA for agent registry and audit logs.** | **P0** |
| 9 | **`INF-009`** ⭐ | **Tenant Isolation Module** | Module | Rust | **NEW: Enforces tenant boundaries at engine startup. Validates tenant_id on all operations.** | **P0** |

---

### Tier 2: Core Framework Crates

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 10 | `FWD-001` | `rapidagent-ir` | Library | Rust | Intermediate Representation types: `Graph`, `Node`, `Edge`, `Reducer`, `State`, `Policy`. Shared across crates. | P0 |
| 11 | `FWD-002` | `rapidagent-envelope` | Library | Rust + TS + Python | EHP Envelope v1 schema: `EHPEnvelope` with codegen targets. JSON Schema, Zod, Prost definitions. | P0 |
| 12 | `FWD-003` | `rapidagent-events` | Library | Rust | Event emitter/subscriber for EHP. NATS-backed in prod, in-process in tests. | P0 |
| 13 | `FWD-004` | `rapidagent-tools` | Library | Python (PyO3) | Tool registry, dispatch, MCP bridge, trust-level evaluation. | P0 |
| 14 | `FWD-005` | `rapidagent-sdk` | SDK | Python + TypeScript | Developer-facing API for building agents: `AgentSpec`, `Tool`, `Skill`, `Policy`. | P1 |
| 15 | **`FWD-006`** | **`ragc` CLI** | Tool | Rust | **CONSOLIDATED: Unified CLI for compile, inspect, run, deploy. Replaces separate CLI components.** | **P1** |
| 16 | **`FWD-007`** | **Cross-Process Comm** | Library | Rust | **NEW: Abstractions for inter-process RPC (NATS req/rep, Unix domain sockets, in-process). Enables Warden/Engine decoupling.** | **P1** |
| 17 | **`FWD-008`** ⭐ | **Telemetry Schema** | Library | Rust | **NEW: Metrics definitions for OpenMetrics/Prometheus. Standardized metric names, types, descriptions.** | **P1** |
| 18 | **`FWD-009`** ⭐ | **NATS Subject Constants** | Library | Rust + TS + Python | **NEW: Type-safe subject builders for all NATS subjects. Eliminates string duplication. Codegen from spec.** | **P0** |

---

### Tier 3: Compiler Suite

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 19 | `CMP-001` | `rw-syspro-compiler` | Crate (existing) | Rust | Lexer → parser → optimizer → renderer pipeline for `.rag` specs. | P0 |
| 20 | `CMP-002` | DAG Validator | Module | Rust | Validates graph topology: no cycles (except explicit loops), required fields, policy well-formedness. | P0 |
| 21 | `CMP-003` | Policy Compiler | Module | Rust | Compiles AgentSpec `policy` section into `policy.json`: allowlists, RBAC rules, resource limits. | P0 |
| 22 | `CMP-004` | System Prompt Renderer | Module | Rust | Renders `system_prompt.md` with variable substitution, skill injection, greeting composition. | P1 |
| 23 | **`CMP-005`** | **Artifact Signer** | Module | Rust | **PRIORITY RAISED: Generates Ed25519 signatures on `.rag` → binary. Provenance via uuidv5(SHA256(payload) + pubkey). Enterprise requirement.** | **P1** |
| 24 | `CMP-006` | Binary Renderer | Module | Rust | Optional `.rapidagent` binary format: section layout, encryption (age/rage), self-verifying header. | P2 |
| 25 | **`CMP-007`** ⭐ | **Version Compatibility Checker** | Module | Rust | **NEW: Validates compiler/engine/framework version compatibility at compile/run time. Prevents version skew.** | **P1** |
| 26 | **`CMP-008`** ⭐ | **Schema Registry** | Service | Rust | **NEW: Stores EHP envelope schemas with versioning. Backward compatibility enforcement. Deprecation warnings.** | **P2** |

---

### Tier 4: Engine & Execution

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 27 | `ENG-001` | `rapidagent-engine` | Crate | Rust | Graph execution engine: DAG scheduler, node evaluators, reducer composition, state management. | P0 |
| 28 | `ENG-002` | Checkpoint Manager | Module | Rust | Saves/restores agent state at node boundaries. JSON + optional binary. | P0 |
| 29 | `ENG-003` | Event Bus Client | Wrapper | Rust | Thin wrapper around `rapidagent-events` + NATS JetStream subscription. | P0 |
| 30 | `ENG-004` | Policy Hook | Middleware | Rust | Pre/post hooks for tool calls, network requests, filesystem access. Calls Warden synchronously via FWD-007. | P0 |
| 31 | `ENG-005` | Message Injector | Service | Rust | Receives POL injection messages from NATS → injects into agent context as `role: "pol"`. | P0 |
| 32 | `ENG-006` | React Loop | Runtime | Rust + Python | LLM call → tool dispatch → state update loop. Python bindings for LLM integration. | P0 |
| 33 | `ENG-007` | Session Manager | Service | Rust | Lifecycle: `CREATED → INITIALIZING → RUNNING → STOPPING → TERMINATED`. | P1 |
| 34 | **`ENG-008`** ⭐ | **Offline Fallback Executor** | Service | Rust | **NEW: Runs engine in single-process mode when NATS unavailable. Local event queue with replay on reconnect.** | **P1** |
| 35 | **`ENG-009`** ⭐ | **Graceful Shutdown Handler** | Service | Rust | **NEW: Drains NATS subscriptions, flushes checkpoints, closes WAL on SIGTERM. Prevents data loss.** | **P1** |
| 36 | **`ENG-010`** ⭐ | **Checkpoint WAL** | Module | Rust | **NEW: Write-ahead log format for crash recovery. Append-only, checkpointed per node boundary.** | **P0** |
| 37 | **`ENG-011`** ⭐ | **Reducer Registry** | Module | Rust | **NEW: Pluggable reducer system. User-defined reducers registered at compile time. Versioned.** | **P1** |

---

### Tier 5: POL Control Plane

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 38 | `POL-001` | `POLControlPlane` | Service | Rust | Daemon component: intervention CRUD, policy evaluation, failure detection, escalation. | P0 |
| 39 | `POL-002` | Policy Evaluator | Module | Rust | Rule-based evaluation: path allowlists, endpoint allowlists, tool trust levels, memory protection. | P0 |
| 40 | `POL-003` | Intervention Store | Storage | Rust + PG | Persistent intervention records: `interventions.json` on disk, sync to PG for HA. | P1 |
| 41 | `POL-004` | POL Subscriber | Service | Rust | NATS subscriber to `pol.{tenant}.events`: detects failures, violations, escalations. | P0 |
| 42 | `POL-005` | WebSocket Stream | API | Rust | Real-time intervention updates to clients. | P1 |
| 43 | `POL-006` | gRPC Status | API | Rust | `AgentControl.Status()` stream: status, metrics, log tail. | P1 |
| 44 | `POL-007` | Human Escalation | Service | Rust | Last-resort delegation: logs alert, notifies admin via Telegram/Discord, waits for approval. | P2 |
| 45 | **`POL-008`** ⭐ | **Session Inspector** | Service | Rust | **NEW: Reads agent session history and live context for POL Agent analysis. Used in Phase 2.** | **P2** |
| 46 | **`POL-009`** ⭐ | **Local Intervention Journal** | Storage | Rust | **NEW: Append-only JSON journal for interventions. Syncs to PG when available. Survives PG outage.** | **P1** |
| 47 | **`POL-010`** ⭐ | **Intervention Priority Enum** | Type | Rust | **NEW: `critical|high|medium|low` enum with cross-language codegen. Used in intervention schema.** | **P1** |

---

### Tier 6: Security Warden

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 48 | `WRD-001` | `SecurityWarden` | Service | Rust | Blocking RBAC enforcement gate. Subscribes to Warden request subject via FWD-007. | P0 |
| 49 | `WRD-002` | Request Interceptor | Middleware | Rust | **SPLIT: Separate interceptors for `fs.read`, `fs.write`, `fs.delete`, `tool.execute`, `network.request`, `memory.delete`.** | P0 |
| 50 | `WRD-003` | Permission Matrix | Data | Rust | Principal type × permission scope × resource type → allow/deny decision. | P0 |
| 51 | `WRD-004` | Audit Logger | Service | Rust | All Warden decisions logged with timestamp, principal, resource, decision, reason. | P1 |
| 52 | `WRD-005` | Fail-Closed Handler | Config | Rust | Timeout or error → deny. Configurable grace period for first-boot (30s default). | P1 |
| 53 | **`WRD-006`** ⭐ | **Circuit Breaker** | Middleware | Rust | **NEW: Detects Warden unhealthiness, fails closed after N consecutive failures. Prevents deadlock.** | **P1** |
| 54 | **`WRD-007`** ⭐ | **PrincipalType Enum** | Type | Rust | **NEW: `USER|AGENT|SYSTEM|ADMIN` enum with cross-language codegen (Rust/Python/TS).** | **P1** |

---

### Tier 7: Daemon & Orchestration

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 55 | `DRN-001` | `rapidagent-daemon` | Service | Rust | Systemd service: spawns/manages engine instances, hosts POL + Warden, exposes gRPC/WS. | P0 |
| 56 | `DRN-002` | Spawn Manager | Module | Rust | `Spawn(SpawnRequest)` → allocates engine, subscribes to NATS, returns `agent_id`. | P0 |
| 57 | `DRN-003` | Health Probe | API | Rust | `/health`, `/ready`, NATS connectivity, component status. | P1 |
| 58 | `DRN-004` | Metrics Exporter | Service | Rust | Prometheus metrics: events/sec, intervention count, Warden decisions, engine uptime. | P1 |
| 59 | **`DRN-005`** | **Config Watcher** | Service | Rust | **DEFERRED: Hot-reload policy/Warden rules. SIGHUP + restart sufficient for MVP.** | **P3** |
| 60 | **`DRN-006`** ⭐ | **Blue-Green Deploy Script** | Tool | Python | **NEW: Zero-downtime daemon updates. Switch traffic after health check passes.** | **P2** |
| 61 | **`DRN-007`** ⭐ | **Rolling Update Manager** | Tool | Rust | **NEW: Safe multi-VM daemon updates with rollback on failure.** | **P2** |
| 62 | **`DRN-008`** ⭐ | **Graceful Shutdown** | Module | Rust | **NEW: Signal handler (SIGTERM/SIGINT) → drain NATS, flush checkpoints, close WAL.** | **P1** |

---

### Tier 8: Observability & Telemetry

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 63 | `OBS-001` | Distributed Tracing | Library | Rust (OTel) | OpenTelemetry integration: trace_id, span, baggage on all EHP envelopes. | P1 |
| 64 | `OBS-002` | Structured Logger | Library | Rust (tracing) | JSON logs with correlation IDs, component tags, severity. | P1 |
| 65 | `OBS-003` | Log Aggregator | Service | Rust/Python | NATS subscriber to telemetry subject → aggregates → stores in PG or Loki. | P2 |
| 66 | **`OBS-004`** | **Dashboard** | UI | TypeScript | **DEFERRED: Grafana/Loki stack for real-time agent health. Operational tooling, not framework.** | **P3** |
| 67 | **`OBS-005`** ⭐ | **Trace Correlation Engine** | Service | Rust | **NEW: OTel collector with cross-component span correlation. Jaeger/Tempo dashboard backend.** | **P2** |
| 68 | **`OBS-006`** ⭐ | **OperationalContext Propagator** | Middleware | Rust | **NEW: Propagates `operationalContext` from EHP envelope through engine → logs → metrics.** | **P1** |

---

### Tier 9: Memory & State

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 69 | **`MEM-001`** | **Honcho Integration** | Library | Rust/Python | **PRIORITY DOWNGRADED: Knowledge graph overlay for agent memory. External dependency, not core.** | **P2** |
| 70 | **`MEM-002`** | **In-Process Deriver** | Service | Rust | **REMOVED: Honcho fork already implements this. Use Honcho directly.** | **—** |
| 71 | **`MEM-003`** | **Dream Cycle** | Service | Rust/Python | **PRIORITY DOWNGRADED: Memory consolidation is Horizon 2+. Nice-to-have for MVP.** | **P3** |
| 72 | **`MEM-004`** | **Rate Limiter** | Utility | Rust | **FOLDED: Token-bucket limiter absorbed into Honcho Integration or removed.** | **—** |

---

### Tier 10: Developer Experience

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 73 | `DX-001` | `ragc` CLI | Tool | Rust | Unified CLI: `ragc compile`, `ragc inspect`, `ragc run`, `ragc deploy`, `ragc status`. | P1 |
| 74 | **`DX-002`** | **IDE Extension** | Plugin | TypeScript | **DEFERRED: VS Code extension for `.rag` syntax. Separate product, not framework core.** | **P3** |
| 75 | `DX-003` | Example Agents | Samples | YAML | Hello-world, file-observer, web-researcher, code-reviewer agents. | P1 |
| 76 | `DX-004` | Template Library | Scaffold | Python/Rust | `ragc init <template>`: starter `.rag` files for common patterns. | P2 |

---

### Tier 11: Security & Compliance

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 77 | `SEC-001` | Secret Manager | Service | Rust (Hashicorp Vault) | API keys, model credentials, signing keys. Not stored in `.rag`. | P1 |
| 78 | `SEC-002` | Audit Trail | Storage | Rust + PG | Immutable log of all interventions, Warden decisions, policy changes. | P1 |
| 79 | `SEC-003` | Key Rotation | Operation | Manual/Script | Ed25519 key pairs for artifact signing. Rotation procedure documented. | P2 |
| 80 | **`SEC-004`** | **Compliance Report** | Report | Python | **REMOVED: SOC2/ISO27001 evidence collection. Phase 4. Track in roadmap only.** | **—** |
| 81 | **`SEC-005`** ⭐ | **Key Storage Policy** | Policy | Rust | **NEW: HSM/Vault required for signing keys. Never store in filesystem or env vars.** | **P1** |
| 82 | **`SEC-006`** ⭐ | **NATS Security Config** | Config | YAML | **NEW: Per-subject ACLs, JWT auth, TLS mandatory. Prevents unauthenticated event injection.** | **P1** |
| 83 | **`SEC-007`** ⭐ | **Agent Sandboxing** | Policy | Linux | **NEW: Seccomp/AppArmor profiles, network namespace isolation, filesystem jailing.** | **P1** |
| 84 | **`SEC-008`** ⭐ | **Policy GitOps** | Service | Rust | **NEW: Git-based policy versioning + automatic deployment to Warden. Declarative security.** | **P2** |
| 85 | **`SEC-009`** ⭐ | **Key Rotation Automation** | Script | Python | **NEW: Automated Ed25519 key rotation with zero-downtime key switching.** | **P2** |

---

### Tier 12: CI/CD & Deployment

| # | Component ID | Name | Type | Language | Description | Priority |
|---|-------------|------|------|----------|-------------|----------|
| 86 | `CD-001` | GitHub Actions CI | Pipeline | YAML | ruff, mypy, cargo test, coverage, security audit on every PR. | P1 |
| 87 | `CD-002` | Release Automation | Script | Python | Semantic versioning, changelog generation, crates.io publish, PyPI publish. | P2 |
| 88 | `CD-003` | Infra Deploy | Ansible | YAML | Playbook for NATS, Postgres, Caddy, daemon systemd service on `infra` VM. | P2 |
| 89 | `CD-004` | Worktree Worker | Plugin | Python | Hermes plugin for isolated git worktrees on `dev` VM. | P1 |
| 90 | `CD-005` | Blue-Green Deploy | Script | Python | **MOVED FROM DRN-006: Zero-downtime daemon updates.** | **P2** |
| 91 | `CD-006` | Rolling Update Manager | Tool | Rust | **MOVED FROM DRN-007: Safe multi-VM daemon updates with rollback.** | **P2** |
| 92 | **`CD-007`** ⭐ | **Cross-Crate Compatibility Test** | CI Job | YAML | **NEW: Tests all crate version combinations in CI. Prevents dependency hell.** | **P1** |
| 93 | **`CD-008`** ⭐ | **Schema Compatibility Test** | CI Job | Rust | **NEW: Validates EHP envelope backward compatibility on schema changes.** | **P1** |

---

## Summary Statistics

| Metric | v1.0 | v2.0 | Change |
|--------|------|------|--------|
| **Total Components** | 62 | **93** | +31 |
| **P0 Components** | 25 | **35** | +10 |
| **P1 Components** | 22 | **34** | +12 |
| **P2 Components** | 16 | **21** | +5 |
| **P3 Components** | 1 | **5** | +4 |
| **Removed** | 0 | **5** | -5 |
| **New Components** | 0 | **36** | +36 |
| **Deferred Components** | 0 | **7** | +7 |

---

## Changes from v1.0 → v2.0

### Added (36 new components)
- **Infrastructure HA**: `INF-007` NATS HA, `INF-008` PG Replica, `INF-009` Tenant Isolation
- **Core Framework**: `FWD-006` consolidated CLI, `FWD-007` Cross-Process Comm, `FWD-008` Telemetry Schema, `FWD-009` NATS Subject Constants
- **Compiler**: `CMP-007` Version Checker, `CMP-008` Schema Registry
- **Engine**: `ENG-008` Offline Fallback, `ENG-009` Graceful Shutdown, `ENG-010` Checkpoint WAL, `ENG-011` Reducer Registry
- **POL**: `POL-008` Session Inspector, `POL-009` Local Intervention Journal, `POL-010` Intervention Priority Enum
- **Warden**: `WRD-006` Circuit Breaker, `WRD-007` PrincipalType Enum
- **Daemon**: `DRN-006` Blue-Green Deploy, `DRN-007` Rolling Update Manager, `DRN-008` Graceful Shutdown
- **Observability**: `OBS-005` Trace Correlation Engine, `OBS-006` OperationalContext Propagator
- **Security**: `SEC-005` Key Storage Policy, `SEC-006` NATS Security Config, `SEC-007` Agent Sandboxing, `SEC-008` Policy GitOps, `SEC-009` Key Rotation Automation
- **CI/CD**: `CD-007` Cross-Crate Compatibility Test, `CD-008` Schema Compatibility Test

### Removed (5 components)
- `MEM-002` In-Process Deriver (use Honcho directly)
- `MEM-004` Rate Limiter (folded into Honcho)
- `SEC-004` Compliance Report (Phase 4)
- `DX-002` IDE Extension (deferred to Horizon 4)
- `OBS-004` Dashboard (deferred to Horizon 3)

### Deferred (7 components)
- `CMP-006` Binary Renderer (Horizon 3)
- `DRN-005` Config Watcher (MVP uses SIGHUP)
- `MEM-001` Honcho Integration (P2, not core)
- `MEM-003` Dream Cycle (Horizon 2+)
- `DX-002` IDE Extension (Horizon 4)
- `OBS-004` Dashboard (Horizon 3)
- `SEC-004` Compliance Report (Horizon 4)

### Priority Adjustments
- `CMP-005` Artifact Signer: P2 → **P1** (enterprise requirement)
- `MEM-001` Honcho Integration: P1 → **P2** (external dependency)
- `MEM-003` Dream Cycle: P2 → **P3** (nice-to-have)
- `ENG-002` Checkpoint Manager: P1 → **P0** (WAL spec makes critical path)

---

## Architecture Simplifications Applied

1. **Consolidated CLI**: `FWD-006` merged with `DX-001` → single `ragc` crate
2. **Unified Observability**: `OBS-002` + `OBS-003` → `rapidagent-telemetry` crate
3. **Simplified Memory**: Removed duplicate Honcho components, kept only integration layer
4. **Deferred Non-Core**: IDE extension, dashboard, compliance reports moved to later horizons
5. **Explicit HA**: Added NATS and PG HA components to eliminate single points of failure
6. **Split Warden Interceptors**: Separate `fs.read` / `fs.write` for different policy semantics
7. **Type-Safe Subjects**: `FWD-009` eliminates string duplication across components

---

## Tier Consolidation (Recommended for Implementation)

While the component list uses 12 tiers for organization, implementation should consolidate to **6 phases**:

| Phase | Tiers Covered | Focus |
|-------|---------------|-------|
| **Phase 1: Foundation** | Tier 1-2 | NATS, PG, IR, Envelope, Events, Subjects |
| **Phase 2: Compiler** | Tier 3 | Parser, validator, policy compiler, signer |
| **Phase 3: Engine** | Tier 4-5 | Graph execution, checkpointing, POL control plane |
| **Phase 4: Governance** | Tier 6-7 | Warden, daemon, health probes |
| **Phase 5: Observability** | Tier 8-9 | Tracing, logging, memory (Honcho) |
| **Phase 6: DX & Security** | Tier 10-12 | CLI, examples, security policies, CI/CD |

---

## Next Steps

1. ✅ Architecture component list v2.0 complete
2. ✅ Forward audit report complete
3. ⏭️ Write SPEC documents for P0 components
4. ⏭️ Write ADRs for key decisions
5. ⏭️ Begin Phase 2 implementation planning

---

*End of Architecture Component List v2.0*