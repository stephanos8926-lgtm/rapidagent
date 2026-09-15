---
title: "RapidAgent Framework Architecture and Design Vision Document"
description: "Comprehensive architectural vision for the RapidAgent Framework — an enterprise-grade AI agent runtime platform by RapidWebs Enterprise"
status: draft
version: 1.0
created: 2026-09-15
author: Lucien (Lead Digital Architect, RapidWebs Enterprise)
reviewers: Steven Page (Lead Dev, RapidWebs Enterprise)
tags:
  - architecture
  - vision
  - rapidagent
  - framework
  - ai-agents
  - runtime
  - pol
  - warden
  - ehp
---

# RapidAgent Framework Architecture and Design Vision Document

## Executive Summary

The **RapidAgent Framework** is a next-generation, enterprise-grade AI agent runtime platform designed to execute, orchestrate, and govern compiled AI agents at production scale. Built by RapidWebs Enterprise, it addresses the fundamental gap between *agent prototyping* and *agent production deployment* by providing a unified stack spanning specification, compilation, execution, observability, and governance.

### Core Philosophy

> **Agents as Software Artifacts** — Not prompts, not scripts, not chat sessions. Agents are *compiled, versioned, signed, auditable software artifacts* with defined interfaces, explicit dependencies, and verifiable behavior.

This philosophy drives every architectural decision: from the `.rag` specification language to the binary `.rapidagent` distribution format, from the NATS-backed Event Horizon Pipeline to the POL Control Plane and Security Warden.

---

## Architectural Vision: Three-Layer Stack

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    RAPIDAGENT RUNTIME FRAMEWORK (Library/SDK/API)                │
│                    pip install rapidagent | cargo add rapidagent                  │
│                    Python + Rust bindings · Type-safe · Observable               │
└─────────────────────────────────────┬───────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        RAPIDWEBS ENGINE (Execution Runtime)                      │
│                    Rust · In-process or Daemon · NATS-backed                     │
│                    Graph execution · Checkpointing · Policy hooks               │
└─────────────────────────────────────┬───────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     COMPILED AGENT ARTIFACT (Output of rw-syspro-compiler)       │
│                    .rag → compiled-agent/ (folder) → .rapidagent (binary)        │
│                    manifest.json · system_prompt.md · graph.ir.json · policy.json │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer | Technology | Responsibility |
|-------|------------|----------------|
| **Runtime Framework** | Python (PyO3/maturin) + Rust | SDK, CLI, tool dispatch, React loop, LLM calls, developer experience |
| **Engine** | Rust (tokio, async-graph) | DAG execution, state management, event emission, policy enforcement, checkpointing |
| **Compiler** | Rust (rw-syspro-compiler) | `.rag` parsing, IR generation, optimization, artifact rendering, signing |

---

## Foundational Pillars

### 1. Event Horizon Pipeline (EHP) — The Nervous System

**Purpose**: Standardized, high-frequency, pub/sub message bus establishing the communication spine for all platform components.

**Design Principles**:
- **Schema-first**: Every message conforms to the **EHP Envelope v1** (JSON Schema + Zod + Rust types)
- **Distributed tracing built-in**: `trace_id`, `task_id`, `graph_id`, `node_id`, `worker_id` on every envelope
- **Transport-agnostic**: NATS JetStream for production (durable, replayable, multi-tenant); in-process for testing
- **Priority-ordered**: 0-255 priority field enables real-time preemption
- **Message types**: `COMMAND`, `EVENT`, `TELEMETRY`, `HEARTBEAT`, `ALERT`

**Subject Taxonomy** (NATS):
```
agents.{tenant}.{agent_id}.events      # JetStream: AGENTS_EVENTS (all agent lifecycle events)
agents.{tenant}.{agent_id}.commands    # Control plane → agent (spawn, stop, inject, pause)
agents.{tenant}.{agent_id}.state       # KV: current state snapshot
agents.{tenant}.registry               # KV: agent metadata index
artifacts.{tenant}.{artifact_hash}     # Object Store: compiled agent binaries
pol.{tenant}.interventions             # Stream: POL interventions & escalations
warden.{tenant}.requests               # Stream: Warden blocking requests (sync)
```

**EHP Envelope v1 Structure**:
```json
{
  "id": "uuidv4",
  "type": "COMMAND|EVENT|TELEMETRY|HEARTBEAT|ALERT",
  "timestamp": 1699999999999,
  "priority": 128,
  "source": { "id": "string", "type": "USER|AGENT|SYSTEM|ADMIN", "version": "string?" },
  "destination": "string",
  "topic": "string",
  "payload": {},
  "context": { "requestId": "string", "workspaceId": "string", "sessionId": "string?", "operationalContext": "string?" },
  "auth": { "uid": "string", "role": "string", "permissions": ["string"], "emailVerified": "boolean?" },
  "signature": "string?"
}
```

### 2. Platform Orchestration Layer (POL) — The Brain

**Purpose**: The primary intelligence and control plane for the entire service layer stack. Not a chat agent — a **daemon service** with an optional LLM-backed *POL Agent* for analysis and remediation.

**POL Control Plane (Daemon — Rules-Based, Always Running)**:
- **Intervention Management**: Create, track, resolve, escalate system interventions
- **Policy Enforcement**: Evaluate execution/network/memory/tool policies via `PolicyEvaluator`
- **Failure Detection**: Subscribe to `nexus.>` events; detect `worker.failed`, `tool.denied`, `task.failed`, `violation`
- **Agent Control**: Cancel tasks, trigger retries, escalate to human (last resort)
- **Real-time Injection**: Inject structured `POLMessage` into active agent sessions
- **Observability**: WebSocket streaming, gRPC status, health endpoints

**POL Agent (LLM-Backed, Phase 2)**:
- **Session Inspection**: Read agent session logs (historical + live)
- **Root Cause Analysis**: Correlate events, identify patterns, propose fixes
- **Remediation Planning**: Generate corrective action plans for POL Control Plane execution
- **Human Escalation**: Only when autonomous resolution confidence < threshold
- **Communication**: **Strict JSON schema** — `POLMessage` types: `intervention`, `guidance`, `command`, `context_update`, `escalation`

**POL Message Format** (injected into agent context as `role: "pol"`):
```json
{
  "type": "intervention|guidance|command|context_update|escalation",
  "intervention_id": "intv-abc123",
  "priority": "critical|high|medium|low",
  "message": "Structured guidance for the agent",
  "action_required": "continue|pause|retry|abort|escalate",
  "context_delta": { "key": "value" },
  "metadata": { "source": "POL", "timestamp": "ISO8601" }
}
```

### 3. Security Warden — The Shield

**Purpose**: RBAC enforcement gate operating as a **blocking synchronous check** on the EHP. Sub-module of POL, owned by POL Agent.

**Architecture**:
- **Blocking Request/Reply**: `bus.request(envelope, timeout)` → Warden evaluates → `bus.respond(requestId, allowed)`
- **Fail-closed**: Timeout or error = deny
- **Intercepts**: `fs.read`, `fs.write`, `fs.delete`, `tool.execute`, `network.request`, `memory.delete`
- **Policy Source**: Compiled from AgentSpec `policy` section + tenant RBAC + platform defaults
- **Principal Types**: `USER`, `AGENT`, `SYSTEM`, `ADMIN` with permission scopes

**Warden Decision Flow**:
```
Agent/Service → EHP request (topic: "tool.execute") 
    → Warden intercepts (blocking)
    → Evaluates: principal, permissions, resource, policy
    → Allows: respond(true) → execution proceeds
    → Denies: respond(false) + audit log → execution blocked
```

---

## Agent Lifecycle & Execution Model

### Compilation Pipeline

```
.rag (YAML frontmatter + DAG spec)
    │
    ▼
┌─────────────────────────────────────┐
│         rw-syspro-compiler           │
│  lexer → parser → optimizer → renderer│
└─────────────────────────────────────┘
    │
    ▼
compiled-agent/ (folder — PRIMARY OUTPUT)
├── manifest.json          # Metadata, version, signatures, dependencies
├── system_prompt.md       # Rendered system prompt with variable substitution
├── graph.ir.json          # Optimized DAG IR (nodes, edges, reducers, state schema)
├── tools.json             # Tool declarations, schemas, trust levels
├── policy.json            # Compiled RBAC, allowlists, resource limits
├── greetings.json         # Conversation starters, personas
└── skills/                # Embedded skill definitions (optional)
    │
    ▼ (future)
.rapidagent (binary — SECONDARY OUTPUT)
├── Ed25519 signature (uuidv5 from SHA256(payload) + compiler pubkey)
├── Optional encryption (age/rage)
└── Self-verifying on load
```

### Runtime Execution

```
┌────────────────────────────────────────────────────────────────┐
│                    rapidagent-daemon (systemd)                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────┐  │
│  │ Spawn Mgr   │ │ POL Ctrl    │ │  Warden     │ │ NATS     │  │
│  │ (pod/ssh/   │ │ Plane       │ │ (blocking   │ │ Client   │  │
│  │  local)     │ │ (interven-  │ │  request)   │ │          │  │
│  └──────┬──────┘ │  tions)     │ └──────┬──────┘ └────┬────┘  │
│         │        └──────┬──────┘        │           │       │
│         ▼               ▼               ▼           ▼       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           Engine Instance (per agent)                   │  │
│  │  ┌────────┐ ┌──────────┐ ┌────────┐ ┌────────┐ ┌─────┐  │  │
│  │  │ Graph  │ │Checkpoint│ │ Event  │ │ Policy │ │Inject│  │  │
│  │  │ Exec   │ │ Manager  │ │ Bus    │ │ Hook   │ │ Msg  │  │  │
│  │  └────────┘ └──────────┘ └────────┘ └────────┘ └─────┘  │  │
│  └────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### Agent Execution Semantics

- **One Engine Instance = One Agent Lifetime** — no sharing, no pooling at engine level
- **Daemon spawns engines** via gRPC `Spawn(SpawnRequest)` → returns `agent_id`
- **Checkpointing**: Automatic at DAG node boundaries + manual via POL command
- **Event Emission**: Every state transition → EHP event (nats subject `agents.{tenant}.{agent_id}.events`)
- **Policy Hooks**: Pre-tool, post-tool, pre-network, pre-fs — all blocking via Warden
- **Message Injection**: POL → `agents.{tenant}.{agent_id}.commands` (topic: `pol.inject`) → Engine delivers to agent context as `role: "pol"`

---

## Multi-Tenancy & Isolation

**Current**: Single-tenant with `tenant_id` namespace on all subjects, KV keys, artifact paths.
**Future**: Full multi-tenant with:
- Per-tenant NATS streams/KV buckets (JetStream)
- Per-tenant PostgreSQL schemas (Honcho, agent registry)
- Per-tenant Warden policy namespaces
- Cross-tenant artifact sharing via signed manifests

---

## Infrastructure Topology (Production)

| Component | Location | Technology |
|-----------|----------|------------|
| **NATS JetStream** | `infra` VM (100.79.58.118) | Port 4222, JetStream enabled, TLS |
| **PostgreSQL 16** | `infra` VM | Honcho (memory), agent registry, audit logs |
| **Caddy** | `infra` VM | TLS termination, reverse proxy |
| **rapidagent-daemon** | `infra` VM (systemd) | Rust binary, gRPC + WebSocket |
| **Dev Worktrees** | `dev` VM (100.109.15.31) | Isolated git worktrees, per-task venvs |
| **Jules** | Google Cloud | Pro Gemini, 15 PRs/day, for heavy refactoring |

---

## Key Design Decisions (ADRs to Be Written)

| # | Decision | Status |
|---|----------|--------|
| 001 | NATS JetStream as primary event transport | ✅ Accepted |
| 002 | gRPC as control plane protocol (OpenAPI supplemental) | ✅ Accepted |
| 003 | `.rag` folder output primary, `.rapidagent` binary secondary | ✅ Accepted |
| 004 | Ed25519 signatures with uuidv5 provenance for binary artifacts | ✅ Accepted |
| 005 | Single-tenant with `tenant_id` namespace (multi-tenant later) | ✅ Accepted |
| 006 | Blocking Warden via NATS request/reply (fail-closed) | ✅ Accepted |
| 007 | POL Control Plane as daemon service (not agent) | ✅ Accepted |
| 008 | POL Agent as compiled RapidAgent artifact (Phase 2) | 🔄 Proposed |
| 009 | Rust Engine + Python Framework via PyO3/maturin | ✅ Accepted |
| 010 | Monorepo with crates: framework, engine, compiler, daemon, warden | ✅ Accepted |
| 011 | EHP Envelope v1 as canonical event schema (codegen targets) | 🔄 Proposed |
| 012 | `policy` field in IR schema compiled from AgentSpec | 🔄 Proposed |
| 013 | Distributed tracing fields mandatory on all events | 🔄 Proposed |

---

## First Workload: Hello-Daemon Agent

**Goal**: Minimal viable agent proving the full stack:
1. `.rag` spec → compiles to `compiled-agent/`
2. Daemon spawns engine via gRPC
3. Engine executes graph, emits events to NATS
4. Warden intercepts tool call (allows)
5. POL detects event, creates intervention (optional)
6. POL injects message → appears in agent context

**Spec** (to be written):
```yaml
# examples/hello-daemon.rag
---
name: hello-daemon
version: 0.1.0
tenant: default
graph:
  nodes:
    - id: greet
      type: llm_call
      config:
        prompt: "Say hello to {{name}}"
      outputs: [message]
    - id: emit
      type: emit_event
      config:
        topic: "demo.greeting"
        payload: "{{message}}"
  edges:
    - from: greet
      to: emit
policy:
  tools: [emit_event]
  network: [nats://infra:4222]
  fs: []
---
```

---

## Success Criteria (MVP)

| Criterion | Metric |
|-----------|--------|
| **Compile** | `.rag` → `compiled-agent/` in <5s |
| **Spawn** | gRPC `Spawn` → engine running in <2s |
| **Execute** | Hello-daemon DAG completes in <10s |
| **Events** | All transitions visible in NATS stream |
| **Warden** | Tool call blocked/allowed in <100ms |
| **Injection** | POL message appears in agent context |
| **Observability** | WebSocket + gRPC health + metrics |

---

## Roadmap Horizons

### Horizon 1: Foundation (Months 1-2) — *Current Focus*
- [ ] Monorepo scaffold with all crates
- [ ] EHP Envelope v1 schema + codegen (Rust, Python, TS)
- [ ] IR schema with `policy` section
- [ ] `rapidagent-warden` crate (blocking NATS request/reply)
- [ ] `rapidagent-daemon` with POL Control Plane + gRPC + WebSocket
- [ ] Engine: graph execution + checkpointing + event emission + injection
- [ ] Hello-daemon end-to-end test

### Horizon 2: Intelligence (Months 3-4)
- [ ] POL Agent as compiled RapidAgent artifact
- [ ] Session inspection + root cause analysis
- [ ] Autonomous remediation planning
- [ ] Human escalation workflow
- [ ] Dream Cycle / memory consolidation integration

### Horizon 3: Platform (Months 5-6)
- [ ] Multi-tenant isolation
- [ ] Binary artifact format (`.rapidagent`) + signing
- [ ] Artifact registry + distribution
- [ ] Fleet management (agent scaling, updates, rollbacks)
- [ ] Policy-as-code GitOps workflow

### Horizon 4: Ecosystem (Months 7+)
- [ ] Language SDKs (TypeScript, Go, Python)
- [ ] MCP server for agent tooling
- [ ] Marketplace for compiled agents
- [ ] Compliance certifications (SOC2, ISO27001)

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| NATS operational complexity | Medium | High | Start simple; managed NATS option; comprehensive runbooks |
| Warden blocking latency | Low | High | Benchmark <100ms p99; async pre-check + sync verify |
| POL Agent hallucination | Medium | High | Rules-first; LLM only for analysis; human gate on actions |
| Compiler/Engine version skew | Medium | Medium | Semantic versioning in manifest; compatibility matrix |
| Distributed tracing adoption | High | Medium | Mandatory tracing fields; auto-instrumentation in SDK |
| Multi-machine sync (workstation ↔ server) | Medium | Medium | `hermes-fork-sync` skill; CI validation on both |

---

## References & Inspiration

| Project | Relevant Concepts |
|---------|-------------------|
| **NexusAgent** (internal) | NATS event bus, POL Control Plane, memory system, runtime foundation, worktree dispatch |
| **Gemini-coder** (internal) | EHP Bus, OrchestrationLayer, SecurityWarden, blocking request/reply |
| **Honcho** (fork) | Memory system, knowledge graph, in-process deriver, embedding transport |
| **rw-syspro-compiler** | Spec compilation, IR pipeline, optimization, rendering |
| **Temporal.io** | Durable execution, checkpointing, replay |
| **Dapr** | Sidecar pattern, pub/sub, state management, actors |
| **LangGraph** | Graph-based agent execution, state reducers |
| **OpenTelemetry** | Distributed tracing, metrics, logging standards |

---

## Glossary

| Term | Definition |
|------|------------|
| **EHP** | Event Horizon Pipeline — the standardized message bus |
| **POL** | Platform Orchestration Layer — control plane daemon |
| **POL Agent** | LLM-backed intelligence backing the POL (Phase 2) |
| **Warden** | Security Warden — blocking RBAC enforcement gate |
| **IR** | Intermediate Representation — compiler output (graph.ir.json) |
| **AgentSpec** | `.rag` file — source specification for an agent |
| **Compiled Agent** | Output artifact (folder or binary) ready for execution |
| **Engine** | Rust execution runtime (per-agent instance) |
| **Daemon** | System service managing engine lifecycle, POL, Warden |
| **Framework** | Python/Rust SDK for developers building agents |
| **Tenant** | Logical isolation namespace (future multi-tenant) |

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-09-15 | Lucien | Initial draft from architecture discussions |

---

*End of Vision Document*