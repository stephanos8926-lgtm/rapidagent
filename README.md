# RapidAgent Framework

> **Enterprise-grade agent orchestration platform with automated governance and security.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![NATS](https://img.shields.io/badge/nats-JetStream-green.svg)](https://nats.io/)
[![PostgreSQL](https://img.shields.io/badge/postgres-16%2B-teal.svg)](https://www.postgresql.org/)

---

## Overview

RapidAgent is a production-ready framework for building, deploying, and governing autonomous AI agents at enterprise scale. It provides:

- **🔒 Security First**: Blocking RBAC enforcement via Security Warden
- **🎯 Governance**: Platform Orchestration Layer (POL) for intervention management
- **⚡ Performance**: Sub-millisecond event transport via NATS JetStream
- **📊 Observability**: OpenTelemetry tracing, structured logging, Prometheus metrics
- **🏗️ Multi-tenant**: Isolated agents with shared infrastructure

The framework implements the **Event Horizon Pipeline (EHP)** architecture with three pillars:

1. **EHP** — Event-driven communication with durable replay
2. **POL** — Autonomous policy enforcement and intervention
3. **Warden** — Synchronous RBAC security gate

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RapidAgent Framework                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│  │   Compiler   │    │    Engine    │    │     POL      │                 │
│  │              │───▶│              │◀───│ Control      │                 │
│  │ • .rag spec  │    │ • DAG exec   │    │   Plane      │                 │
│  │ • Validator  │    │ • Checkpoints│    │ • Interventions│                │
│  │ • Policy     │    │ • React loop │    │ • Escalation │                 │
│  │   Compiler   │    │ • Sessions   │    │              │                 │
│  └──────────────┘    └──────┬───────┘    └──────┬───────┘                 │
│                             │                   │                         │
│                             └────────┬──────────┘                         │
│                                      │                                     │
│              ┌───────────────────────┼───────────────────────┐            │
│              │                       ▼                       │            │
│              │              ┌─────────────────┐              │            │
│              │              │  Security       │              │            │
│              │              │  Warden         │              │            │
│              │              │  • RBAC         │              │            │
│              │              │  • Audit Log    │              │            │
│              │              │  • Fail-closed  │              │            │
│              │              └────────┬────────┘              │            │
│              │                       │                       │            │
│              └───────────────────────┼───────────────────────┘            │
│                                      │                                     │
│        ┌─────────────────────────────┼─────────────────────────────┐      │
│        │                             ▼                             │      │
│        │              ┌─────────────────────────┐                  │      │
│        │              │   Infrastructure        │                  │      │
│        │              │   • NATS JetStream      │                  │      │
│        │              │   • PostgreSQL 16       │                  │      │
│        │              │   • Caddy (TLS)         │                  │      │
│        │              │   • Tailscale Mesh      │                  │      │
│        │              └─────────────────────────┘                  │      │
│        │                                                             │      │
│        └─────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Components

### Tier 1: Infrastructure
| Component | Purpose |
|-----------|---------|
| NATS JetStream Cluster | Event transport with durable streams |
| PostgreSQL 16 | Persistent storage for agents, interventions, audit |
| Caddy Reverse Proxy | TLS termination, gRPC/WS routing |
| Tailscale Mesh | Private networking across VMs |

### Tier 2: Core Framework
| Component | Purpose |
|-----------|---------|
| `rapidagent-ir` | Intermediate representation types |
| `rapidagent-envelope` | EHP message schema (Rust + TS + Python) |
| `rapidagent-events` | Event emitter/subscriber |
| `rapidagent-tools` | Tool registry and dispatch |
| `rapidagent-sdk` | Developer-facing API |
| `ragc` CLI | Compile, inspect, run, deploy agents |

### Tier 3: Compiler Suite
| Component | Purpose |
|-----------|---------|
| `rw-syspro-compiler` | Lexer → parser → optimizer → renderer |
| DAG Validator | Graph topology validation |
| Policy Compiler | Compiles policy to JSON |
| Artifact Signer | Ed25519 signatures on artifacts |

### Tier 4: Engine & Execution
| Component | Purpose |
|-----------|---------|
| `rapidagent-engine` | Graph execution engine |
| Checkpoint Manager | State save/restore |
| Policy Hook | Pre/post hooks for Warden |
| Message Injector | POL context injection |
| React Loop | LLM → tool → state loop |

### Tier 5: POL Control Plane
| Component | Purpose |
|-----------|---------|
| `POLControlPlane` | Intervention CRUD, escalation |
| Policy Evaluator | Rule-based evaluation |
| Intervention Store | Persistent intervention records |
| Human Escalation | Telegram/Discord notifications |

### Tier 6: Security Warden
| Component | Purpose |
|-----------|---------|
| `SecurityWarden` | Blocking RBAC enforcement |
| Request Interceptor | File/tool/network checks |
| Permission Matrix | Principal × permission → decision |
| Audit Logger | All decisions logged |

---

## Quick Start

### Prerequisites

- Rust 1.75+
- Python 3.11+
- PostgreSQL 16+
- NATS Server 2.10+
- Hashicorp Vault (for secrets)

### Installation

```bash
# Clone the repository
git clone https://github.com/rapidwebs/rapidagent.git
cd rapidagent

# Build the workspace
cargo build --release

# Set up PostgreSQL
psql -U postgres -c "CREATE DATABASE rapidagent;"
psql -U postgres -d rapidagent -c "CREATE EXTENSION vector;"

# Configure NATS
nats-server --jetstream --store_dir /var/lib/nats

# Generate configuration
ragc init --template hello-world --output ./my-agent/
```

### First Agent

Create `hello.rag`:

```yaml
name: hello-world
version: 1.0.0
policy:
  allow_paths:
    - /tmp/**
  deny_tools:
    - rm
graph:
  start: hello
  nodes:
    hello:
      type: python
      code: |
        print("Hello from RapidAgent!")
```

Compile and run:

```bash
ragc compile hello.rag --output ./compiled/
ragc run ./compiled/ --agent-id test-001
```

---

## Documentation

- **[Architecture Vision](docs/plans/RapidAgent Framework Architecture and Design Vision Document.md)** — Complete vision document
- **[Component List](docs/plans/architecture-component-list.md)** — 93 components across 12 tiers
- **[Data Storage Architecture](docs/plans/data-storage-architecture.md)** — Persistence strategy and security
- **[SPEC-001: EHP Envelope](docs/specs/SPEC-001-ehp-envelope.md)** — Message schema
- **[SPEC-002: Warden Blocking](docs/specs/SPEC-002-warden-blocking.md)** — Security gate protocol
- **[SPEC-003: POL Interventions](docs/specs/SPEC-003-pol-interventions.md)** — Governance protocol
- **[ADR-001: NATS JetStream](docs/adrs/ADR-001-nats-jetstream.md)** — Event transport decision
- **[ADR-002: Blocking Warden](docs/adrs/ADR-002-blocking-warden.md)** — Security gate decision
- **[ADR-003: POL Daemon](docs/adrs/ADR-003-pol-daemon.md)** — Control plane decision
- **[ADR-004: Data Storage](docs/adrs/ADR-004-data-storage.md)** — Persistence decision

---

## Roadmap

| Phase | Deliverables | Target |
|-------|--------------|--------|
| **Phase 1** | Foundation (INF, FWD tiers) | Q4 2026 |
| **Phase 2** | Compiler (CMP tier) | Q1 2027 |
| **Phase 3** | Engine + POL (ENG, POL tiers) | Q1 2027 |
| **Phase 4** | Governance (WRD, DRN tiers) | Q2 2027 |
| **Phase 5** | Observability (OBS, MEM tiers) | Q2 2027 |
| **Phase 6** | DX + Security (DX, SEC, CD tiers) | Q3 2027 |

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -am 'feat: add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Create a new Pull Request

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

- NATS.io for event streaming
- PostgreSQL and pgvector for data persistence
- Hashicorp Vault for secrets management
- The Rust and Python communities for their excellent ecosystems

---

*Built by RapidWebs Enterprise*
