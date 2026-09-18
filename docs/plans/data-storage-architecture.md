# Data Storage and Persistence Design Decisions

---
name: data-storage-architecture
description: Comprehensive data classification, persistence strategy, and security requirements for the RapidAgent Framework
status: draft
created: 2026-09-15
author: Lucien
related-adrs:
  - "ADR-001-nats-jetstream.md"
  - "ADR-002-blocking-warden.md"
  - "ADR-003-pol-daemon.md"
---

# Data Storage Architecture — RapidAgent Framework

## Executive Summary

This document establishes the data storage architecture for the RapidAgent Framework, classifying data by persistence lifecycle, security requirements, and access patterns. The architecture uses a **multi-tier storage strategy**: PostgreSQL for structured relational data, NATS JetStream for event streaming, and encrypted filesystem storage for sensitive artifacts.

---

## 1. Data Classification Matrix

### 1.1 By Persistence Lifecycle

| Category | Description | Examples | Retention |
|----------|-------------|----------|-----------|
| **LONG-LIVED** | Persistent data requiring durability and backup | Agent specs, policies, interventions, audit logs | Indefinite + compliance |
| **SHORT-LIVED** | Ephemeral data for runtime operation | Checkpoints, session state, tool intermediates | Process lifetime or crash recovery |
| **TRANSIENT** | Live event streams, no persistent storage | NATS ephemeral messages, in-memory caches | Seconds to minutes |

### 1.2 By Security Sensitivity

| Level | Description | Examples | Requirements |
|-------|-------------|----------|--------------|
| **PUBLIC** | No sensitivity | Compiled binaries, documentation | Integrity only |
| **INTERNAL** | Business logic, not sensitive | Agent definitions, metrics | Access control |
| **CONFIDENTIAL** | Requires protection | Interventions, Warden decisions, policies | Encryption at rest, audit trail |
| **SECRET** | Critical security data | API keys, signing keys, credentials | HSM/Vault, strict RBAC, rotation |
| **TOP-SECRET** | Legal/compliance required | Audit logs, forensic evidence | Immutable, tamper-evident, WORM |

---

## 2. Storage Tier Assignments

### 2.1 PostgreSQL (Structured, Queryable, HA Required)

| Data Type | Category | Sensitivity | Notes |
|-----------|----------|-------------|-------|
| Agent Registry | LONG | INTERNAL | Relational queries, versioning |
| Intervention Store | LONG | CONFIDENTIAL | Transactional integrity |
| Audit Trail | LONG | TOP-SECRET | Immutable, append-only |
| Warden Decisions | LONG | CONFIDENTIAL | Queryable for compliance |
| Policy Config | LONG | CONFIDENTIAL | Versioned, validated |
| Embeddings/Vectors | LONG | INTERNAL | pgvector for semantic search |
| Metrics/Telemetry | SHORT | INTERNAL | Time-series, rollup possible |
| Node Registry | LONG | INTERNAL | Cluster membership |

**Rationale**: PostgreSQL provides:
- ACID transactions for intervention/Warden data
- Replication for HA (streaming replica)
- pgvector for embedding similarity search
- Row-level security for multi-tenancy
- JSONB for flexible schema evolution

### 2.2 NATS JetStream (Event Streaming, Replay)

| Data Type | Category | Sensitivity | Notes |
|-----------|----------|-------------|-------|
| Event Stream | TRANSIENT | INTERNAL | Durable streams with replay |
| Configuration KV | SHORT | INTERNAL | Hot-reloadable configs |
| Agent Heartbeats | SHORT | INTERNAL | Liveness monitoring |
| Telemetry Publisher | SHORT | INTERNAL | High-throughput metrics |
| Intervention Events | SHORT | CONFIDENTIAL | Pub/sub for POL subscribers |

**Rationale**: NATS JetStream provides:
- Sub-millisecond latency
- Built-in replay for debugging
- Interest-based subscription
- Native request/reply pattern
- TLS for in-transit encryption

### 2.3 Filesystem (Artifacts, Keys, Logs)

| Data Type | Category | Sensitivity | Storage Method |
|-----------|----------|-------------|----------------|
| .rag Source Files | LONG | PUBLIC | Git repository |
| Compiled Artifacts | LONG | PUBLIC | Object Store / Filesystem |
| Checkpoint WAL | SHORT | INTERNAL | JSONL append-only |
| Local Intervention Journal | LONG | CONFIDENTIAL | Encrypted JSONL |
| API Keys/Secrets | SECRET | SECRET | Hashicorp Vault (primary) |
| Signing Keys | SECRET | TOP-SECRET | HSM / Vault (primary) |
| Audit Logs | TOP-SECRET | TOP-SECRET | Immutable JSONL + WAL |
| Telemetry Logs | SHORT | INTERNAL | Structured JSON |

**Encryption Methods**:
- **File-level**: `age`/`rage` for individual sensitive files
- **Directory-level**: `fscrypt` or LUKS for entire directories
- **Volume-level**: LUKS/dm-crypt for virtual disk (Horizon 3+)
- **In-transit**: TLS 1.3 for all network connections

---

## 3. Data Lifecycle Management

### 3.1 Write-Ahead Logging (WAL) Pattern

For critical data requiring crash recovery:

```
PostgreSQL Primary
    │
    ├── Write to WAL (disk)
    │   └── ~/.rapidagent/wal/{entity}.jsonl
    │
    ├── Replicate to PG Replica
    │   └── Streaming replication
    │
    └── Synchronize from WAL
        └── Background sync job
```

**Use Cases**:
- Intervention Journal: Local JSONL → sync to PG when available
- Checkpoint WAL: Engine state → recover on crash
- Audit Trail: Immutable log → replicate to cold storage

### 3.2 Secure Deletion Requirements

| Data Type | Deletion Method | Rationale |
|-----------|-----------------|-----------|
| Checkpoints | Secure wipe (`shred`, `blkdiscard`) | Contain transient computation |
| Session State | In-memory only, no disk | No forensic recovery |
| Tool Intermediates | Volatile memory | Ephemeral by design |
| NATS Messages | Stream retention policy | TTL-based cleanup |
| Secrets | Key rotation + Vault deletion | Zero-knowledge proof |

### 3.3 Backup and Replication Strategy

| Data Type | Backup Method | Frequency | Retention |
|-----------|---------------|-----------|-----------|
| PostgreSQL | pg_dump + PITR | Hourly | 30 days |
| Interventions | Logical backup + JSONL | Continuous | Indefinite |
| Audit Trail | Immutable append + offsite | Continuous | 7 years (compliance) |
| Policies | Git + Vault snapshots | On change | Indefinite |
| Signing Keys | HSM backup + Vault | On rotation | Per key lifecycle |

---

## 4. Security Requirements by Data Type

### 4.1 Encryption at Rest

| Data Type | Algorithm | Key Management | Implementation |
|-----------|-----------|----------------|----------------|
| Interventions | AES-256-GCM | Vault-managed | File-level with age |
| Audit Logs | AES-256-GCM | HSM-managed | Immutable JSONL |
| Warden Decisions | AES-256-GCM | Vault-managed | Database column encryption |
| API Keys | N/A (Vault) | Vault automation | Never stored on disk |
| Signing Keys | N/A (HSM) | Hardware security | Raw key never leaves HSM |

### 4.2 File System Permissions

| Path | Owner | Group | Permissions | Rationale |
|------|-------|-------|-------------|-----------|
| `/etc/rapidagent/` | root | rapidagent | 0750 | Config directory |
| `/var/lib/rapidagent/` | rapidagent | rapidagent | 0750 | Data directory |
| `/var/log/rapidagent/` | rapidagent | adm | 0640 | Log files |
| `~/.rapidagent/secrets/` | rapidagent | rapidagent | 0700 | Private keys |
| `~/.rapidagent/vault/` | rapidagent | rapidagent | 0700 | API keys |
| `/opt/rapidagent/artifacts/` | rapidagent | rapidagent | 0755 | Public artifacts |

### 4.3 User and Group Isolation

```
System User: rapidagent (service account)
System Group: rapidagent
Home Directory: /var/lib/rapidagent/
Log Directory: /var/log/rapidagent/
Config Directory: /etc/rapidagent/
Data Directory: /var/lib/rapidagent/data/
```

---

## 5. Virtual Disk Architecture (Horizon 3+)

### 5.1 Concept

An encrypted virtual disk image providing:
- **Full-disk encryption** for sensitive directories
- **Versioning** of disk snapshots
- **Tamper detection** via cryptographic hashing
- **Secure mount/unmount** with daemon lifecycle

### 5.2 Implementation Options

| Option | Technology | Pros | Cons |
|--------|------------|------|------|
| **LUKS/dm-crypt** | Linux native | Battle-tested, kernel-level | Complex setup, single-mount |
| **fscrypt** | Linux filesystem | Transparent encryption | ext4/xfs only |
| **age-encrypted tar** | age/rage | Portable, simple | No random access |
| **Custom volume manager** | Rust | Full control | Development overhead |

### 5.3 Recommended Approach

**Phase 1 (MVP)**: Use age-encrypted archives for sensitive data
```bash
# Create encrypted archive
age -r recipient@example.com -o secrets.age secrets/

# Mount on demand
age -d secrets.age | tar -xzf - -C /tmp/secrets/
```

**Phase 2 (Horizon 2)**: Implement fscrypt for transparent directory encryption
```bash
# Enable fscrypt on data directory
fscrypt setup /var/lib/rapidagent/
fscrypt protect /var/lib/rapidagent/secrets
```

**Phase 3 (Horizon 3)**: Evaluate LUKS virtual disk for critical security workloads

---

## 6. Decision Matrix

### 6.1 Database Selection

| Requirement | SQLite | PostgreSQL | Decision |
|-------------|--------|------------|----------|
| Multi-tenancy | Limited | Full (schemas, RLS) | **PostgreSQL** |
| High availability | None | Streaming replication | **PostgreSQL** |
| Vector search | Extensions (limited) | pgvector (native) | **PostgreSQL** |
| Remote access | File-based only | Network protocols | **PostgreSQL** |
| Concurrency | Weak | Strong | **PostgreSQL** |
| Operations complexity | Simple | Moderate | Trade-off accepted |

**Decision**: PostgreSQL for production, SQLite only for local/dev testing.

### 6.2 NATS JetStream vs Other Message Brokers

| Feature | NATS JetStream | Kafka | RabbitMQ | Redis Streams |
|---------|----------------|-------|----------|---------------|
| Latency | <1ms | 10-100ms | 5-20ms | <1ms |
| Throughput | 1M+ msgs/sec | 1M+ msgs/sec | 100K msgs/sec | 500K msgs/sec |
| Replay | Native | Native | Manual | Limited |
| Complexity | Low | High | Medium | Low |
| Multi-tenant | Subjects | Topics | Exchanges | Keyspaces |
| HA | Cluster mode | Partitioning | Clustering | Sentinel |

**Decision**: NATS JetStream for event transport (already selected in ADR-001).

### 6.3 Secret Storage

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| **Environment variables** | Simple | Leaks in logs, ps | ❌ Reject |
| **Config files** | Versionable | Plaintext risk | ❌ Reject |
| **Hashicorp Vault** | Dynamic secrets, audit, HSM | Operational overhead | ✅ **Select** |
| **AWS KMS** | Managed, scalable | Vendor lock-in | ❌ Reject (self-hosted) |
| **age/rage** | Portable, simple | No key rotation, no audit | ✅ Supplementary |

**Decision**: Hashicorp Vault as primary secret manager, age/rage for file-level encryption.

---

## 7. Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            RapidAgent Data Flow                            │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Client     │     │   Engine     │     │   Warden     │
│   (gRPC/WS)  │────▶│              │────▶│              │
└──────────────┘     └──────┬───────┘     └──────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Runtime Operations                                │
│                                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Session  │  │ Check-   │  │ Tool     │  │ Event    │  │ POL      │    │
│  │ State    │  │ point    │  │ Results  │  │ Stream   │  │ Context  │    │
│  │(volatile)│  │(WAL)     │  │(volatile)│  │(NATS)    │  │(NATS)    │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │             │             │             │             │           │
│       └─────────────┴─────────────┴─────────────┴─────────────┘           │
│                           │                                               │
│                           ▼                                               │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                     PostgreSQL (Persistent)                       │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │    │
│  │  │ Agent      │ │ Intervention│ │ Warden     │ │ Audit      │   │    │
│  │  │ Registry   │ │ Store      │ │ Decisions  │ │ Trail      │   │    │
│  │  │            │ │            │ │            │ │            │   │    │
│  │  │ • Name     │ │ • ID       │ │ • Timestamp│ │ • Events   │   │    │
│  │  │ • Spec     │ │ • Priority │ │ • Decision │ │ • Actor    │   │    │
│  │  │ • Version  │ │ • Guidance │ │ • Reason   │ │ • Result   │   │    │
│  │  │ • Status   │ │ • Status   │ │ • Principal│ │ • Source   │   │    │
│  │  │ • Created  │ │ • Created  │ │ • Allowed  │ │ • Detail   │   │    │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │    │
│  │                                                                  │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐                  │    │
│  │  │ Policies   │ │ Embeddings │ │ Metrics    │                  │    │
│  │  │            │ │ (pgvector) │ │            │                  │    │
│  │  │ • Rules    │ │ • Vectors  │ │ • Events/s │                  │    │
│  │  │ • RBAC     │ │ • Metadata │ │ • Interv   │                  │    │
│  │  │ • Allow    │ │ • Search   │ │ • Warden   │                  │    │
│  │  └────────────┘ └────────────┘ └────────────┘                  │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                     NATS JetStream (Events)                       │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │    │
│  │  │ agents.    │ │ pol.       │ │ warden.    │ │ telemetry. │   │    │
│  │  │ {tenant}.  │ │ {tenant}.  │ │ {tenant}.  │ │ {tenant}.  │   │    │
│  │  │ events     │ │ events     │ │ requests   │ │ events     │   │    │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                     Filesystem (Artifacts)                        │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │    │
│  │  │ .rag Files │ │ Checkpoint │ │ Intervention│ │ Secrets    │   │    │
│  │  │ (Git)      │ │ WAL        │ │ Journal    │ │ (Vault)    │   │    │
│  │  │            │ │ (JSONL)    │ │ (JSONL)    │ │ (age/      │   │    │
│  │  │ • Source   │ │ • State    │ │ • Local    │ │  Vault)    │   │    │
│  │  │ • Version  │ │ • Restore  │ │ • Sync     │ │            │   │    │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │    │
│  └──────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Open Questions and Decisions

| Question | Recommendation | Status |
|----------|----------------|--------|
| Should we use SQLite for local development? | Yes, with migration path to PostgreSQL | ✅ Accepted |
| Should NATS be used for all events or only specific ones? | All inter-component events; PostgreSQL for persistent state | ✅ Accepted |
| Should we implement virtual disk (LUKS) for MVP? | No, use age-encrypted archives; evaluate in Horizon 3 | ✅ Accepted |
| Should secrets be stored in PostgreSQL or Vault? | Vault for secrets; PostgreSQL for non-secret metadata | ✅ Accepted |
| Should audit logs be immutable? | Yes, append-only with cryptographic chaining | ✅ Accepted |
| What is the backup retention policy? | 30 days for operational data, 7 years for compliance | 🔄 To be confirmed |
| Should we implement data tiering (hot/warm/cold)? | Phase 3+; start with single PostgreSQL instance | 🔄 To be confirmed |

---

## 9. References

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [Hashicorp Vault Documentation](https://developer.hashicorp.com/vault/docs)
- [age Encryption Tool](https://filippo.io/age)
- [fscrypt Documentation](https://github.com/google/fscrypt)
- [Linux Unified Key Setup (LUKS)](https://wiki.archlinux.org/title/Dm-crypt)

---

*End of Data Storage Architecture Document*
