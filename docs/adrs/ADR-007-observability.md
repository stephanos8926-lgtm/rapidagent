# ADR-007: Observability Stack

---
title: "ADR-007: Observability Stack"
status: Accepted
date: 2026-09-16
deciders: Lucien, Steven Page
consulted: RapidWebs Infrastructure Team
informed: All contributors

---

## Context

The RapidAgent Framework requires comprehensive observability for:
- Debugging agent executions
- Performance monitoring
- Security auditing
- Cost tracking
- Incident response

Options evaluated: Prometheus + Grafana, Datadog, ELK stack, OpenTelemetry.

## Decision

We will use **OpenTelemetry** with OTLP exporter as the unified observability stack.

**Decision:** Deploy OpenTelemetry Collector with the following pipeline:
- Traces → Jaeger (debugging)
- Metrics → Prometheus (monitoring)
- Logs → Loki (log aggregation)

## Alternatives Considered

| Option | Pros | Cons | Reason Rejected |
|--------|------|------|-----------------|
| **Datadog** | Managed, great UX | Expensive, vendor lock-in | We want self-hosted for cost control |
| **ELK stack** | Powerful, flexible | Heavy resource usage, complex ops | Overkill for our scale |
| **Prometheus + Grafana only** | Lightweight, standard | No distributed tracing, limited log support | Insufficient for agent debugging |
| **OpenTelemetry** ✅ | Vendor-neutral, unified, extensible | Requires setup, learning curve | Best long-term value |

## Consequences

### Positive
- **Vendor-neutral**: Can switch backends without code changes
- **Unified**: Traces, metrics, logs in one stack
- **Standard**: OTLP is emerging standard
- **Rust-native**: Excellent SDK support
- **Cost-effective**: Self-hosted, no per-GB pricing

### Negative
- **Setup complexity**: Requires multiple components
- **Resource usage**: Collector adds ~200MB RAM
- **Learning curve**: Team needs OTel training
- **Debugging**: More moving parts to troubleshoot

### Mitigations
- Use containerized deployment (docker-compose)
- Start with minimal config, expand as needed
- Document common patterns in AGENTS.md

## Implementation Notes

- Deploy OpenTelemetry Collector on infra VM
- Configure receivers: OTLP HTTP, OTLP gRPC
- Configure exporters: Jaeger, Prometheus, Loki
- Add OTel SDK to all crates
- Set up Prometheus scraping for metrics

## References
- [SPEC-006: OpenTelemetry](../specs/SPEC-006-opentelemetry.md)
- [OpenTelemetry Rust Documentation](https://opentelemetry.io/docs/languages/rust/)
