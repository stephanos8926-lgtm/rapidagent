# ADR-001: NATS JetStream as Primary Event Transport

---
title: "ADR-001: NATS JetStream as Primary Event Transport"
status: Accepted
date: 2026-09-15
deciders: Lucien, Steven Page
consulted: RapidWebs Infrastructure Team
informed: All contributors

---

## Context

The RapidAgent Framework requires a high-performance, reliable, and distributed event transport layer to support:
- Real-time agent communication
- Durable event storage for replay
- Multi-tenant message isolation
- Cross-VM coordination (workstation ↔ infra ↔ dev)

Existing solutions evaluated: Kafka, RabbitMQ, Redis Pub/Sub, gRPC streaming, in-process EventEmitter.

## Decision

We will use **NATS JetStream** as the primary event transport for the RapidAgent Framework.

**Decision:** We will deploy NATS JetStream on the `infra` VM (100.79.58.118) with the following configuration:
- Port 4222 (client), 8222 (monitoring)
- JetStream enabled with file-based storage
- TLS enabled for all client connections
- Per-subject ACLs for multi-tenancy
- 3-node cluster for HA (Horizon 2+)

## Alternatives Considered

| Option | Description | Pros | Cons | Reason Rejected |
|--------|-------------|------|------|-----------------|
| **Kafka** | Distributed event streaming | High throughput, durable | Complex ops, heavy footprint, overkill for our scale | NATS is simpler, lighter, sufficient for our needs |
| **RabbitMQ** | AMQP broker | Good tooling, familiar | No native replay, complex topology | NATS has built-in JetStream replay |
| **Redis Pub/Sub** | In-memory pub/sub | Fast, simple | No durability, no replay, single-point-of-failure | JetStream provides durability + replay |
| **gRPC Streaming** | Bidirectional streaming | Native to our stack | No persistence, no broadcast, complex topology | NATS supports pub/sub + persistence |
| **In-Process EventEmitter** | Node.js-style events | Simple, fast | Single process only, no distribution | We need multi-VM coordination |

## Consequences

### Positive
- **Durability**: JetStream persists messages to disk, survives restarts
- **Replay**: Consumers can replay historical events for debugging/recovery
- **Simplicity**: NATS is lightweight (<50MB RAM), easy to deploy
- **Performance**: Sub-millisecond latency, 1M+ msgs/sec
- **Multi-tenancy**: Subject prefixes (`agents.{tenant}.>`) provide isolation
- **Ecosystem**: Mature client libraries for Rust, Python, TypeScript

### Negative
- **Operational complexity**: Requires NATS运维 knowledge (recovered via managed option)
- **Single point of failure** (initially): Single NATS node until Horizon 2 HA cluster
- **Learning curve**: Team needs NATS/JetStream training
- **Debugging**: Distributed tracing required to debug event flow

### Neutral/Follow-ups
- Monitor NATS memory usage and disk I/O
- Plan NATS cluster upgrade for Horizon 2
- Document NATS subject taxonomy in AGENTS.md

## Implementation Notes

- Deploy NATS via systemd on `infra` VM
- Use `nats-server` official image (not Podman unless required)
- Configure JetStream with 10GB storage limit
- Enable monitoring port 8222 for Prometheus scraping
- Document in `docs/research/nats-deployment.md`

## References
- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [NATS Security Guide](https://docs.nats.io/running-a-nats-service/security)
- [RapidWebs Infrastructure Topology](../plans/RapidAgent Framework Architecture and Design Vision Document.md#infrastructure-topology-production)
