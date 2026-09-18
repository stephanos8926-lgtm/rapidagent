# SPEC-006: OpenTelemetry Observability Integration

---
name: OpenTelemetry-Observability
description: Unified telemetry stack with traces, metrics, and logs via OTLP
status: draft
created: 2026-09-16
author: Lucien
related-adrs:
  - "ADR-007"
---

# Spec: OpenTelemetry Observability

## Goal
Implement vendor-neutral observability using OpenTelemetry Rust SDK with OTLP exporter, supporting traces, metrics, and logs.

## Context
Phase 2 research established OpenTelemetry as the standard for unified telemetry:
- Native Rust SDK (opentelemetry v0.28)
- OTLP exporter for Prometheus + Jaeger backends
- Context propagation across service boundaries
- Low overhead (<5% performance impact)

## Requirements

### R1: Tracer Setup

```rust
use opentelemetry::{global, trace::{Tracer, TracerProvider}};
use opentelemetry_sdk::{trace::{Sampler, TracerProvider asSdkTracerProvider}, runtime};
use opentelemetry_otlp::{WithExportConfig};

fn init_tracer() -> TracerProvider {
    let provider =SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(
            Sampler::TraceIdRatioBased(0.1) // 10% sampling
        )))
        .with_config(
            global::Registry::default()
                .tracer_provider()
                .tracer("rapidagent")
        );
    
    global::set_tracer_provider(provider.clone());
    provider
}
```

### R2: Span Convention

| Span Name | Description | Attributes |
|-----------|-------------|------------|
| `rapidagent.agent.spawn` | Agent execution started | `agent_id`, `template`, `workspace_id` |
| `rapidagent.node.execute` | Node execution | `node_id`, `node_type`, `input_hash` |
| `rapidagent.node.complete` | Node execution completed | `node_id`, `latency_ms`, `receipt_hash` |
| `rapidagent.graph.checkpoint` | Checkpoint saved | `graph_id`, `node_id` |
| `rapidagent.policies.evaluate` | Policy evaluation | `policy_id`, `result` |
| `rapidagent.warden.check` | Warden security check | `check_type`, `result` |

### R3: Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `rapidagent.agent.active` | Gauge | Current active agents | `workspace_id` |
| `rapidagent.node.duration` | Histogram | Node execution time | `node_type`, `workspace_id` |
| `rapidagent.graph.checkpoint.size` | Histogram | Checkpoint size | `graph_id` |
| `rapidagent.policies.evaluation` | Counter | Policy evaluations | `policy_id`, `result` |
| `rapidagent.warden.decisions` | Counter | Warden decisions | `decision`, `reason` |

### R4: Log Convention
- Use `tracing` crate with OTel subscriber
- Structured logging with JSON output
- Log levels: TRACE, DEBUG, INFO, WARN, ERROR

### R5: Exporter Configuration
- OTLP HTTP/JSON endpoint: `http://jaeger:4318/v1/traces`
- OTLP gRPC endpoint: `http://prometheus:8889`
- Batch processor with 5s timeout, 2000 max queue

### R6: Testing Requirements
- [ ] Tracer initialization test
- [ ] Span creation and completion test
- [ ] Metric recording test
- [ ] Log export test
- [ ] Context propagation test

## Out of Scope
- Custom exporters
- Distributed tracing UI
- Alerting rules

## References
- [Phase 2 Research: OpenTelemetry Integration](../research/opentelemetry-rust.md)
- [OpenTelemetry Rust Documentation](https://opentelemetry.io/docs/languages/rust/)
