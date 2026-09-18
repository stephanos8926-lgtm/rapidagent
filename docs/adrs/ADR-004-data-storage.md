# ADR-004: Data Storage and Persistence Architecture

---
title: "ADR-004: Data Storage and Persistence Architecture"
status: Accepted
date: 2026-09-15
deciders: Lucien, Steven Page
consulted: Infrastructure Team, Security Team
informed: All contributors

---

## Context

The RapidAgent Framework requires a robust data storage architecture that balances:
- **Durability**: Critical data must survive crashes and restarts
- **Security**: Sensitive data must be encrypted and access-controlled
- **Performance**: Runtime operations require low-latency access
- **Operational simplicity**: minimize infrastructure complexity for MVP

Key questions:
1. PostgreSQL vs SQLite for production data?
2. What data requires encryption at rest?
3. How do we handle secure deletion of ephemeral data?
4. Should we use a virtual disk for sensitive files?

## Decision

We will implement a **multi-tier storage architecture**:

1. **PostgreSQL** (primary persistent store)
   - All structured, queryable data
   - Agent registry, interventions, audit trail, Warden decisions
   - pgvector for embeddings
   - Multi-tenant via schemas and row-level security

2. **NATS JetStream** (event transport)
   - All inter-component events
   - Durable streams with replay
   - Configuration KV pairs
   - High-throughput telemetry

3. **Filesystem** (artifacts and secrets)
   - Source files (.rag) in Git
   - Compiled artifacts in Object Store
   - Sensitive data via Hashicorp Vault or age-encrypted archives
   - Checkpoint WAL in JSONL format

4. **In-Memory** (ephemeral state)
   - Session state, tool intermediates
   - No disk persistence (volatility requirement)

## Alternatives Considered

| Option | Description | Pros | Cons | Reason Rejected |
|--------|-------------|------|------|-----------------|
| **SQLite only** | Single-file database | Simple, portable | No HA, limited concurrency, no remote access | Production needs HA and multi-tenant |
| **PostgreSQL only** | Rely solely on PG | Single technology, powerful | Overkill for ephemeral data, no event replay | Need NATS for event streaming |
| **Flat files only** | All data as JSON/JSONL | Simple, portable | No querying, no transactions, no concurrency | Cannot support relational queries |
| **Virtual disk (LUKS)** | Encrypted block device | Full-disk encryption, versioning | Complex, overkill for MVP | Phase 3+ consideration |
| **Cloud storage** | S3, GCS, etc. | Managed, scalable | Vendor lock-in, latency, cost | Self-hosted requirement |

## Consequences

### Positive
- **PostgreSQL** provides ACID transactions, replication, and pgvector
- **NATS JetStream** provides durable event replay and low latency
- **Filesystem** provides portability and version control (Git)
- **Vault** provides secrets management with HSM support
- Clear separation between persistent and ephemeral data

### Negative
- **Increased operational complexity** (3 storage systems to manage)
- **Data synchronization** required between PostgreSQL and local journals
- **Secret rotation** complexity with Vault integration
- **Backup strategy** must cover multiple storage tiers

### Neutral/Follow-ups
- Monitor PostgreSQL connection pool usage
- Plan Vault deployment strategy (self-hosted vs managed)
- Document backup/restore procedures for each tier

## Implementation Details

### PostgreSQL Schema Design

```sql
-- Tenant isolation via schemas
CREATE SCHEMA tenants.rapidwebs;
CREATE SCHEMA tenants.example_corp;

-- Enable Row-Level Security
ALTER TABLE tenants.rapidwebs.interventions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenants.rapidwebs.interventions
    USING (tenant_id = current_setting('app.current_tenant'));

-- Vector search with pgvector
CREATE EXTENSION vector;
CREATE TABLE embeddings (
    id UUID PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    embedding vector(1536),  -- OpenAI ada-002 dimensions
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX ON embeddings USING ivfflat (embedding vector_cosine_ops);
```

### NATS JetStream Streams

```
Subject Pattern          | Purpose                    | Durable?
-------------------------|----------------------------|----------
agents.{tenant}.{id}.events      | Agent lifecycle events     | Yes
pol.{tenant}.interventions  | POL intervention events    | Yes
warden.{tenant}.requests  | Warden check requests      | Yes
warden.{tenant}.replies   | Warden check responses     | Yes
telemetry.{tenant}.events | High-volume metrics        | No (ephemeral)
config.{tenant}.kv        | Configuration KV pairs     | Yes
```

### Filesystem Layout

```
/etc/rapidagent/           # Config (0750, root:rapidagent)
/var/lib/rapidagent/       # Data (0750, rapidagent:rapidagent)
  ├── postgres/            # PostgreSQL data directory
  ├── natss/               # NATS JetStream data
  ├── checkpoints/         # Checkpoint WAL (0700)
  └── secrets/             # Encrypted secrets (0700)
/var/log/rapidagent/       # Logs (0640, rapidagent:adm)
/opt/rapidagent/artifacts/ # Compiled artifacts (0755)
```

### Secure Deletion

```bash
# Secure wipe for checkpoints (after processing)
shred -n 3 -u /var/lib/rapidagent/checkpoints/session-*.jsonl

# Delete ephemeral NATS streams
nats stream delete TELEMETRY

# Vault secret deletion
vault delete secret/rapidagent/sessions/$SESSION_ID
```

## References
- [Data Storage Architecture Document](./data-storage-architecture.md)
- [ADR-001: NATS JetStream](./ADR-001-nats-jetstream.md)
- [SPEC-003: POL Interventions](../specs/SPEC-003-pol-interventions.md)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Hashicorp Vault Documentation](https://developer.hashicorp.com/vault/docs)
