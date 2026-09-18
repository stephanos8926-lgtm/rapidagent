# Research: NATS JetStream Production HA Patterns

**Date**: 2026-09-15  
**Author**: Lucien (Lead Digital Architect)  
**Status**: Complete  

---

## Executive Summary

NATS JetStream clustering provides production-grade reliability through Raft consensus. Key design decisions: **odd server counts**, **R3 replication**, and **proper resource sizing**. The research reveals common pitfalls (R1 streams on clusters, even server counts) and proven patterns for deployment.

---

## Key Findings

### 1. Cluster Architecture

**Server Count Rule:**
- ✅ **Use odd numbers**: 3 or 5 servers
- ❌ **Never use even numbers**: 2 or 4 servers lose majority protection
- Rationale: Raft quorum = ⌊n/2⌋ + 1. Three servers = 2 needed. Two servers = 1 needed (no protection).

**Meta Group vs Stream Group:**
- **Meta Group**: Coordinates stream/consumer placement across cluster (elects meta leader)
- **Stream Group**: Each stream has its own Raft group (elects stream leader for writes)
- These are independent — stream leader may differ from meta leader

### 2. Replication Strategy

**Replica Factor (R):**
```bash
# Single replica (NOT HA)
nats stream add ORDERS --subjects "orders.>" --replicas 1

# Triple replica (PRODUCTION)
nats stream add ORDERS --subjects "orders.>" --replicas 3
```

**R3 Costs:**
- An R3 stream on an un-tiered account counts as `replicas × bytes` for `MaxStore`
- A 10 GiB ORDERS stream at R3 spends **30 GiB** of account MaxStore
- On tiered accounts, R3 multiplier is baked into tier size

**Critical Pitfall:**
> "A single copy means no failover. Don't assume 'it runs on the cluster' means 'it survives a failure.' Audit replica counts and raise the streams that matter to R3."

### 3. Resource Sizing

**Four Resources to Budget:**

| Resource | Rule | Default Behavior |
|----------|------|------------------|
| **CPU** | 20-30% headroom above steady state | No hard limit; JetStream replication is CPU-intensive |
| **Memory** | ~128 MiB per light client | Memory storage defaults to 75% of RAM |
| **Disk** | Pin `max_file_store` to volume size | File storage defaults to 75% of disk (or 1 TB fallback) |
| **FDs** | 2 per stream + routes + gossip | Default 1024; raise to 800000 for large clusters |

**Configuration Example:**
```yaml
jetstream:
  store_dir: /nats/storage
  max_memory_store: -1  # Use disk
  max_file_store: 10GiB  # Pin to volume size

cluster:
  name: C1
  listen: 0.0.0.0:6222
  routes:
    - nats://host-b:6222
    - nats://host-c:6222
```

### 4. Deployment Patterns

**Kubernetes StatefulSet:**
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: nats
spec:
  replicas: 3
  serviceName: nats
  template:
    metadata:
      labels:
        app: nats
    spec:
      containers:
      - name: nats
        image: nats:2.10
        command:
          - nats-server
          - '-js'
          - '-cid'
          - C1
          - '-hn'
          - '0.0.0.0'
          - '-P'
          - '/etc/nats-config/nats.conf'
        volumeMounts:
        - name: nats-config
          mountPath: /etc/nats-config
        - name: nats-data
          mountPath: /var/lib/nats/data
  volumeClaimTemplates:
  - metadata:
      name: nats-data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

**Rolling Upgrade Procedure:**
1. Set lame-duck mode on node-to-upgrade
2. Upgrade version
3. Verify stream availability
4. Proceed to next node

### 5. Operational Commands

**Audit Replica Counts:**
```bash
# List all R1 streams (non-HA)
nats stream ls --filter=replicas=1

# Audit a specific stream
nats stream info ORDERS --json | jq '.state.replicas'
```

**Health Checks:**
```bash
# Check peer count matches expected replicas
nats server info n1-east --server tls://nats.acme.internal:4222 --creds /etc/nats/creds/sys.creds

# Expected: "OK ORDERS OK:3 peers"
```

**Configuration Reload:**
```bash
# SIGHUP reload without dropping connections
kill -HUP $(pidof nats-server)
```

---

## Application to RapidAgent

### Infrastructure Requirements

| Component | Specification | Justification |
|-----------|---------------|---------------|
| **NATS Cluster Size** | 3 servers (minimum) | Odd count for Raft quorum |
| **Stream Replicas** | R3 for all production streams | Survive single node failure |
| **Placement** | Spread across AZs/racks | Independent failure domains |
| **Memory** | 4 GiB per node minimum | JetStream + client connections |
| **Disk** | 100 GiB SSD minimum | File storage for streams |
| **FD Limit** | 800000 | Avoid exhaustion with multiple streams |

### NATS Subject Design for RapidAgent

```
# Agent lifecycle events (R3 stream)
agents.{tenant}.{agent_id}.events

# Control plane commands (R3 stream)
pol.{tenant}.commands

# Warden requests (R3 stream)
warden.{tenant}.{agent_id}.requests

# Audit trail (R3 stream, immutable)
audit.{tenant}.events

# Telemetry (R1 acceptable - metrics only)
telemetry.{tenant}.events
```

### Tenant Isolation via Accounts

```yaml
accounts:
  rapidagent_prod:
    jetstream: true
    max_memory_store: -1
    max_file_store: 10GiB
    max_streams: 100
    max_consumers: 500
  rapidagent_dev:
    jetstream: true
    max_memory_store: 256MiB
    max_file_store: 1GiB
    max_streams: 10
    max_consumers: 50
```

---

## References

- [NATS JetStream Clustering Documentation](https://docs.nats.io/learn/topologies/jetstream-in-a-cluster)
- [NATS Deployment & Upgrades Guide](https://docs.nats.io/learn/deployment/)
- [NATS Sizing & Resources](https://docs.nats.io/learn/deployment/sizing-and-resources)
- [NATS Clustering Deep Dive](https://github.com/nats-io/nats.docs/blob/master/running-a-nats-service/configuration/clustering/jetstream_clustering/README.md)

---

*Research completed: 2026-09-15*  
*Next steps: Update INF-001 component spec with these patterns*
