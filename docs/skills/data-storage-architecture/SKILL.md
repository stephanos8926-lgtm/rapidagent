---
name: data-storage-architecture
description: Comprehensive data classification, persistence strategy, and security requirements for the RapidAgent Framework
status: active
version: 1.0
category: architecture
triggers:
  - "data storage"
  - "persistence"
  - "encryption at rest"
  - "secret management"
keywords:
  - postgresql
  - nat
  - jetstream
  - vault
  - age
  - encryption
  - backup
  - replication
---

# Data Storage Architecture

Comprehensive guide for data storage, persistence, and security in the RapidAgent Framework.

## Quick Reference

| Data Type | Storage | Encryption | Backup | Retention |
|-----------|---------|------------|--------|-----------|
| Agent Specs | PostgreSQL + Git | Optional | Yes | Indefinite |
| Interventions | PostgreSQL + JSONL | AES-256-GCM | Yes + replicate | 7 years |
| Audit Trail | PostgreSQL | AES-256-GCM | Yes + offsite | 7 years |
| Warden Decisions | PostgreSQL | Column encryption | Yes | Indefinite |
| Secrets/Keys | Hashicorp Vault | HSM | Minimal | Per rotation |
| Checkpoints | Filesystem (JSONL) | Optional | No | Crash recovery |
| NATS Events | JetStream | TLS only | No | Stream retention |

## Storage Tiers

### Tier 1: PostgreSQL (Persistent, Queryable)
- Agent registry
- Intervention store
- Audit trail
- Warden decisions
- Policy configs
- Embeddings (pgvector)
- Metrics/telemetry

### Tier 2: NATS JetStream (Event Streaming)
- Inter-component events
- Configuration KV
- Agent heartbeats
- Telemetry publisher
- POL intervention events

### Tier 3: Filesystem (Artifacts & Secrets)
- Source files (.rag) in Git
- Compiled artifacts
- Checkpoint WAL (JSONL)
- Local intervention journal
- Secrets via Vault or age

## Security Requirements

### Encryption at Rest
- **Interventions**: AES-256-GCM via age-encrypted JSONL
- **Audit logs**: Immutable, hashed chain
- **Warden decisions**: PostgreSQL column encryption
- **Secrets**: Hashicorp Vault (never on disk)

### File Permissions
```
/etc/rapidagent/           0750 root:rapidagent
/var/lib/rapidagent/       0750 rapidagent:rapidagent
/var/lib/rapidagent/secrets/ 0700 rapidagent:rapidagent
/var/log/rapidagent/       0640 rapidagent:adm
```

### Secure Deletion
- Checkpoints: `shred -n 3 -u`
- NATS streams: TTL-based cleanup
- Secrets: Vault deletion + key rotation

## Decision Matrix

### PostgreSQL vs SQLite
| Factor | PostgreSQL | SQLite | Decision |
|--------|------------|--------|----------|
| Multi-tenancy | Full (schemas, RLS) | Limited | PostgreSQL |
| HA | Streaming replication | None | PostgreSQL |
| Vector search | pgvector native | Extension | PostgreSQL |
| Remote access | Network protocols | File-based | PostgreSQL |
| Concurrency | Strong | Weak | PostgreSQL |

**Verdict**: PostgreSQL for production, SQLite only for local dev.

### Secret Storage
| Option | Verdict |
|--------|---------|
| Environment variables | ❌ Reject (leaks in logs) |
| Config files | ❌ Reject (plaintext risk) |
| Hashicorp Vault | ✅ Select (primary) |
| age/rage | ✅ Supplementary (file-level) |

## Virtual Disk (Horizon 3+)

### Phase 1: MVP
- age-encrypted archives for sensitive data
- File-level encryption with age

### Phase 2: Horizon 2
- fscrypt for transparent directory encryption
- Evaluate LUKS for critical workloads

### Phase 3: Horizon 3
- Full LUKS virtual disk evaluation
- Versioned disk images with tamper detection

## Backup Strategy

| Data | Method | Frequency | Retention |
|------|--------|-----------|-----------|
| PostgreSQL | pg_dump + PITR | Hourly | 30 days |
| Interventions | Logical + JSONL | Continuous | Indefinite |
| Audit trail | Immutable + offsite | Continuous | 7 years |
| Policies | Git + Vault | On change | Indefinite |
| Signing keys | HSM backup | On rotation | Per lifecycle |

## References
- [Data Storage Architecture Document](../plans/data-storage-architecture.md)
- [ADR-004: Data Storage and Persistence](../adrs/ADR-004-data-storage.md)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [Hashicorp Vault Documentation](https://developer.hashicorp.com/vault/docs)
