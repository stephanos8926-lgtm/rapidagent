# Research: OpenTelemetry Integration in Rust

**Date**: 2026-09-15  
**Author**: Lucien (Lead Digital Architect)  
**Status**: Complete  

---

## Executive Summary

OpenTelemetry Rust provides production-grade instrumentation for traces, metrics, and logs. Key libraries: `opentelemetry`, `opentelemetry-sdk`, `opentelemetry-otlp` (exporter), and `opentelemetry-appender-tracing` (logs bridge). This research outlines integration patterns for RapidAgent's observability layer.

---

## Core Libraries

### Primary Crates

| Crate | Purpose | Version |
|-------|---------|---------|
| `opentelemetry` | API (traces, metrics, logs) | 0.28.0 |
| `opentelemetry-sdk` | SDK implementation | 0.28.0 |
| `opentelemetry-otlp` | OTLP exporter (HTTP/gRPC) | 0.28.0 |
| `opentelemetry-appender-tracing` | Bridge tracing → OTel logs | 0.28.0 |
| `opentelemetry-prometheus` | Prometheus metrics export | 0.28.0 |
| `tracing` | Structured logging API | 0.1 |
| `tracing-subscriber` | Subscriber implementation | 0.3 |

### Dependency Configuration

```toml
[dependencies]
opentelemetry = { version = "0.28", features = ["metrics", "trace"] }
opentelemetry_sdk = { version = "0.28", features = ["trace", "metrics", "logs"] }
opentelemetry-otlp = { version = "0.28", features = ["grpc-tonic", "reqwest-client"] }
opentelemetry-appender-tracing = "0.28"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry", "env-filter"] }
```

---

## Implementation Pattern

### 1. Provider Initialization

```rust
use opentelemetry::global;
use opentelemetry_sdk::{trace::SdkTracerProvider, metrics::SdkMeterProvider, logs::SdkLoggerProvider};
use opentelemetry_otlp::{WithExportConfig};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;

fn init_tracer_provider() -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::newExporter()
        .http()
        .with_endpoint("http://otel-collector:4318/v1/traces")
        .build()
        .unwrap();
    
    SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(config_with_service_name("rapidagent-daemon"))
        .build()
}

fn init_meter_provider() -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::newExporter()
        .http()
        .with_endpoint("http://otel-collector:4318/v1/metrics")
        .build()
        .unwrap();
    
    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build()
}

fn init_logger_provider() -> SdkLoggerProvider {
    SdkLoggerProvider::builder()
        .with_simple_exporter(opentelemetry_otlp::newExporter()
            .http()
            .with_endpoint("http://otel-collector:4318/v1/logs")
            .build()
            .unwrap())
        .build()
}
```

### 2. Tracing Integration

```rust
use tracing_subscriber::{prelude::*, EnvFilter};

fn setup_tracing(logger_provider: &SdkLoggerProvider) {
    // Create OTel bridge for logs
    let otel_layer = OpenTelemetryTracingBridge::new(logger_provider);
    
    // Configure filtering
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(otel_layer)
        .init();
}
```

### 3. Span Creation

```rust
use opentelemetry::trace::{Tracer, SpanKind, Status};
use opentelemetry::KeyValue;

fn execute_agent_turn(tracer: &impl Tracer, agent_id: &str, turn_id: &str) {
    let mut span = tracer
        .span_builder("agent.turn.execute")
        .with_kind(SpanKind::Internal)
        .start(tracer);
    
    span.set_attribute(KeyValue::new("agent.id", agent_id));
    span.set_attribute(KeyValue::new("turn.id", turn_id));
    
    // Execute turn logic...
    
    span.set_status(Status::Ok);
    span.end();
}
```

### 4. Metrics Collection

```rust
use once_cell::sync::OnceLock;
use opentelemetry::metrics::{Meter, Counter};

static TURN_COUNTER: OnceLock<Counter<u64>> = OnceLock::new();

fn get_turn_counter(meter: &Meter) -> &'static Counter<u64> {
    TURN_COUNTER.get_or_init(|| {
        meter.u64_counter("rapidagent.turns.total")
            .with_description("Total agent turns executed")
            .build()
    })
}

// Usage:
let counter = get_turn_counter(&meter);
counter.add(1, &[
    KeyValue::new("agent.id", agent_id),
    KeyValue::new("status", "success"),
]);
```

---

## Integration with RapidAgent Components

### Agent Control Plane

```rust
// rapidagent-daemon/src/tracing.rs
pub struct AgentTracer {
    tracer: BoxedTracer,
    meter: Meter,
}

impl AgentTracer {
    pub fn spawn_agent(&self, tenant_id: &str, agent_id: &str) {
        let span = self.tracer.span("agent.spawn").start(&self.tracer);
        span.set_attribute(KeyValue::new("tenant.id", tenant_id));
        span.set_attribute(KeyValue::new("agent.id", agent_id));
        span.end();
    }
    
    pub fn execute_turn(&self, agent_id: &str, turn_id: &str) -> impl Drop {
        let span = self.tracer.span("agent.turn").start(&self.tracer);
        span.set_attribute(KeyValue::new("agent.id", agent_id));
        span.set_attribute(KeyValue::new("turn.id", turn_id));
        AgentSpanGuard(span)
    }
}

struct AgentSpanGuard(opentelemetry::trace::Span);

impl Drop for AgentSpanGuard {
    fn drop(&mut self) {
        self.0.end();
    }
}
```

### Event Horizon Pipeline

```rust
// rapidagent-engine/src/ehp/observer.rs
use opentelemetry::trace::{Tracer, SpanKind};

pub struct EHPObserver {
    tracer: BoxedTracer,
}

impl EHPObserver {
    pub fn on_envelope_received(&self, envelope: &EHPEnvelope) {
        let span = self.tracer
            .span_builder("ehp.envelope.receive")
            .with_kind(SpanKind::Consumer)
            .start(&self.tracer);
        
        span.set_attribute(KeyValue::new("envelope.type", envelope.envelope_type.to_string()));
        span.set_attribute(KeyValue::new("tenant.id", envelope.tenant_id.clone()));
        span.set_attribute(KeyValue::new("trace.id", envelope.trace_id.to_string()));
        
        span.end();
    }
    
    pub fn on_envelope_published(&self, envelope: &EHPEnvelope, subject: &str) {
        let span = self.tracer
            .span_builder("ehp.envelope.publish")
            .with_kind(SpanKind::Producer)
            .start(&self.tracer);
        
        span.set_attribute(KeyValue::new("subject", subject));
        span.set_attribute(KeyValue::new("priority", envelope.priority as u64));
        
        span.end();
    }
}
```

### Checkpoint Manager

```rust
// rapidagent-engine/src/checkpoint/observer.rs
use opentelemetry::metrics::{Meter, Histogram};

pub struct CheckpointMetrics {
    meter: Meter,
    duration_histogram: Histogram<f64>,
    size_histogram: Histogram<u64>,
}

impl CheckpointMetrics {
    pub fn record_checkpoint(&self, duration: std::time::Duration, size: u64) {
        self.duration_histogram.record(
            duration.as_secs_f64(),
            &[KeyValue::new("checkpoint.type", "agent_state")],
        );
        self.size_histogram.record(size as f64, &[]);
    }
}
```

---

## Exporter Options

### OTLP (Recommended for Production)

```rust
use opentelemetry_otlp::{WithExportConfig, TonicConfig};

let exporter = opentelemetry_otlp::new_exporter()
    .tonic()
    .with_endpoint("https://otel-collector.rapidwebs.internal:4317")
    .with_timeout(Duration::from_secs(10))
    .build_grpc_exporter()
    .expect("Failed to create OTLP exporter");
```

### Prometheus (For Metrics)

```rust
use opentelemetry_prometheus;

let prometheus_exporter = opentelemetry_prometheus::exporter()
    .with_sender(Default::default())
    .build()?;

let meter_provider = SdkMeterProvider::builder()
    .with_periodic_exporter(prometheus_exporter)
    .build();
```

### Console (For Development)

```rust
use opentelemetry_stdout::{SpanExporter, LogExporter, MetricExporter};

let tracer_provider = SdkTracerProvider::builder()
    .with_simple_exporter(SpanExporter::default())
    .build();
```

---

## Context Propagation

### Trace Context Across NATS

```rust
use opentelemetry::propagation::{Injector, Extractor};
use opentelemetry_jaeger_propagator::JaegerPropagator;

// Inject trace context into NATS metadata
struct NatsInjector(Vec<(String, String)>);

impl Injector for NatsInjector {
    fn set(&mut self, key: &str, value: String) {
        self.0.push((key.to_string(), value));
    }
}

// Extract trace context from NATS metadata
struct NatsExtractor(pub HashMap<String, String>);

impl Extractor for NatsExtractor {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
    
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|s| s.as_str()).collect()
    }
}

// Usage in EHP
pub fn inject_trace_context(envelope: &mut EHPEnvelope, ctx: &Context) {
    let mut injector = NatsInjector(vec![]);
    ctx.propagator().inject_context(ctx, &mut injector);
    
    for (key, value) in injector.0 {
        envelope.metadata.insert(key, value);
    }
}
```

---

## Sampling Strategies

```rust
use opentelemetry_sdk::trace::{Sampler, TracerProvider};

// Production: Sample 10% of requests
let provider = SdkTracerProvider::builder()
    .with_config(
        Config::default()
            .with_sampler(Sampler::ParentBased(Box::new(
                Sampler::TraceIdRatioBased(0.1)
            )))
            .with_resource(Resource::new(vec![
                KeyValue::new("service.name", "rapidagent-daemon"),
                KeyValue::new("deployment.environment", "production"),
            ]))
    )
    .build();
```

---

## Best Practices

### 1. Service Name Consistency

Always set `service.name` resource attribute:
```rust
Resource::new(vec![
    KeyValue::new("service.name", "rapidagent-daemon"),
    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    KeyValue::new("host.name", hostname::get()?.to_string_lossy().into()),
])
```

### 2. Error Attributes

Mark spans as error when appropriate:
```rust
span.set_status(Status::error(format!("Execution failed: {}", e)));
span.add_event("error", vec![KeyValue::new("error.message", e.to_string())]);
```

### 3. Avoid Over-Instrumentation

- Instrument at component boundaries, not every function call
- Use metrics for quantitative data, traces for qualitative flow
- Keep span names consistent and short

### 4. Shutdown Sequence

```rust
// Flush all telemetry before exit
tracer_provider.shutdown()?;
meter_provider.shutdown()?;
logger_provider.shutdown()?;
```

---

## References

- [OpenTelemetry Rust Documentation](https://opentelemetry.io/docs/languages/rust/)
- [OpenTelemetry Getting Started](https://opentelemetry.io/docs/languages/rust/getting-started/)
- [OpenTelemetry GitHub](https://github.com/open-telemetry/opentelemetry-rust/)
- [Rust tracing Documentation](https://docs.rs/tracing)

---

*Research completed: 2026-09-15*  
*Next steps: Add OpenTelemetry dependencies to rapidagent-daemon and rapidagent-engine crates*
