# Research: gRPC + WebSocket Daemon Architecture Patterns

**Date**: 2026-09-15  
**Author**: Lucien (Lead Digital Architect)  
**Status**: Complete  

---

## Executive Summary

Modern daemon architectures use **dual-interface patterns**: gRPC for control plane (typed, fast) and WebSocket for data plane (streaming, real-time). This research examines production patterns from Bud (Rust daemon + Node.js service) and Velocity-Dex (high-frequency trading engine) to inform RapidAgent's daemon design.

---

## Pattern 1: Bud Architecture (Rust Daemon + Node.js Service)

### Architecture Diagram

```
┌─────────────────┐         ┌─────────────────────────────────────┐         ┌─────────────────┐
│   Bud Daemon    │ WS/gRPC │            Service                  │  HTTP   │     Web UI      │
│   (Rust CLI)    │◀───────▶│         (Node.js/Fastify)           │◀───────▶│   (React/Vite)  │
│                 │         │                                     │   SSE   │                 │
└────────┬────────┘         └──────────────────┬──────────────────┘         └─────────────────┘
         │                                     │
         │ Terminal backend                    │ SQL
         ▼                                     ▼
   [tmux/session]                           [PostgreSQL]
```

### Communication Protocols

| Path | Protocol | Purpose |
|------|----------|---------|
| Daemon ↔ Service | WebSocket binary `BudEnvelope` baseline, optional HTTP/2 gRPC control/data | Device reauth, heartbeat, terminal control |
| Daemon → Service | HTTP REST | Device-claim bootstrap (`/api/device-auth/start`, `/api/device-auth/poll`) |
| Service ↔ Web UI | HTTP REST | CRUD operations for buds, threads, messages, sessions |
| Service → Web UI | SSE | Real-time streaming of agent events, terminal output |
| Service → LLM Provider | HTTP | LLM inference via OpenAI Responses API, Anthropic Messages API |

### Technology Stack

**Bud Daemon (Rust):**
| Component | Library | Purpose |
|-----------|---------|---------|
| Async runtime | `tokio` | Concurrent I/O |
| WebSocket client | `tokio-tungstenite` | Bidirectional communication |
| gRPC client | `tonic` + `prost` | Optional control plane |
| CLI parsing | `clap` | Argument parsing |
| PTY handling | `nix` | Terminal sessions |
| TLS | `rustls` | Secure connections |

**Service (Node.js):**
| Component | Library | Purpose |
|-----------|---------|---------|
| HTTP server | `fastify` | REST API |
| WebSocket | `@fastify/websocket` | Daemon connections |
| gRPC | `@grpc/grpc-js` | Optional control plane |
| SSE | `fastify-sse-v2` | Real-time updates |
| Auth | `better-auth` | Browser authentication |
| ORM | `drizzle` | Type-safe database access |
| Database | `postgresql` | Primary storage |
| Logging | `pino` | Structured logging |

### Key Design Decisions

1. **Transport-neutral data plane** — WebSocket baseline with optional gRPC for control
2. **Device-claim bootstrap** — Out-of-band authentication via HTTP REST
3. **Terminal backend abstraction** — tmux adapter allows future PTY/mosh backends
4. **Protocol evolution** — Proto definitions enable future HTTP/2/QUIC upgrades

---

## Pattern 2: Velocity-Dex (High-Frequency Trading Engine)

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      api-server crate                           │
├─────────────────────┬───────────────────────────────────────────┤
│   gRPC Server       │   WebSocket Server                        │
│   (Port 50051)      │   (Port 3000)                             │
│                     │                                           │
│  ┌───────────────┐  │  ┌───────────────┐                       │
│  │ Order APIs    │  │  │ Market Data   │                       │
│  │ (place/cancel)│  │  │ (trade updates)│                       │
│  └───────┬───────┘  │  └───────┬───────┘                       │
│          │          │          │                               │
│          ▼          │          ▼                               │
│  ┌───────────────┐  │  ┌───────────────┐                       │
│  │ MarketProcessor│◄─┴──┤ Event Queue   │                       │
│  │ (Actor Model) │  │  │               │                       │
│  └───────┬───────┘  │  └───────┬───────┘                       │
│          │          │          │                               │
│          ▼          │          ▼                               │
│  ┌───────────────┐  │  ┌───────────────┐                       │
│  │ Orderbook     │  │  │ WAL Handler   │                       │
│  │ (in-memory)   │  │  │ (persistence) │                       │
│  └───────────────┘  │  └───────────────┘                       │
└─────────────────────┴───────────────────────────────────────────┘
```

### Technology Stack

| Component | Library | Purpose |
|-----------|---------|---------|
| Async runtime | `tokio` | High-performance I/O |
| gRPC | `tonic` + `prost` | Trading API |
| WebSocket | `axum` + `tokio-tungstenite` | Market data feed |
| Serialization | `serde` + `bincode` | Binary encoding for low latency |
| Actor model | `tokio::sync::mpsc` | Single-threaded processor |
| Persistence | `tokio::fs` + `bincode` | WAL for crash recovery |
| Metrics | `hdrhistogram` | Latency distribution tracking |

### Actor Model Pattern

```rust
use tokio::sync::mpsc;

// MarketProcessor runs single-threaded, eliminating mutex contention
let (tx, mut rx) = mpsc::channel::<Order>(1024);

// Spawn processor task
tokio::spawn(async move {
    let mut orderbook = Orderbook::new();
    
    while let Some(order) = rx.recv().await {
        match order.action {
            OrderAction::Place(o) => {
                orderbook.place(o).await;
                // Publish to WebSocket subscribers
                market_data.broadcast(OrderUpdate::Placed(o)).await;
            }
            OrderAction::Cancel(id) => {
                orderbook.cancel(id).await;
                market_data.broadcast(OrderUpdate::Cancelled(id)).await;
            }
        }
    }
});
```

### Performance Characteristics

- **4k+ TPS** on dual-core hardware
- **Sub-millisecond P99 latency** for order processing
- **Zero-copy deserialization** via bincode
- **Deterministic execution** via single-threaded actor

---

## Pattern 3: Protocol Multiplexing (Single Port)

### Architecture

Traditional microservices run gRPC and REST on separate ports, requiring:
- Multiple health checks
- Multiple TLS certificates
- Complex load balancer configuration

Protocol multiplexing handles gRPC, gRPC-Web, and REST on one port:

```rust
pub async fn build_app(config: &Config) -> Result<(Router, SocketAddr)> {
    // gRPC services
    let grpc_routes = Routes::new(health_service)
        .add_service(auth_service)
        .add_service(user_service);
 
    // REST routes
    let rest_router = rest_routes(state);
 
    // gRPC-Web layer enables browsers (HTTP/1.1)
    let grpc_router = grpc_routes
        .into_axum_router()
        .layer(GrpcWebLayer::new());
 
    let app = rest_router.merge(grpc_router).layer(middleware);
    Ok((app, addr))
}
```

### Protocol Detection

Requests are routed based on the `Content-Type` header:
- `application/grpc` → native gRPC (mobile/desktop clients)
- `application/grpc-web` → gRPC-Web (browser clients)
- `application/json` → REST (health checks, webhooks)

### Benefits

1. **One port to expose** — Simplifies firewall rules
2. **One TLS certificate** — Single cert management
3. **One health check** — Unified monitoring
4. **One middleware stack** — Consistent cross-cutting concerns

---

## Application to RapidAgent

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    rapidagent-daemon                            │
├─────────────────────┬───────────────────────────────────────────┤
│   gRPC Server       │   WebSocket Server                        │
│   (Port 50051)      │   (Port 8080)                             │
│                     │                                           │
│  ┌───────────────┐  │  ┌───────────────┐                       │
│  │ AgentControl  │  │  │ Event Stream  │                       │
│  │ (Spawn/Stop)  │  │  │ (status/logs) │                       │
│  └───────┬───────┘  │  └───────┬───────┘                       │
│          │          │          │                               │
│          ▼          │          ▼                               │
│  ┌───────────────┐  │  ┌───────────────┐                       │
│  │ AgentManager  │  │  │ Event Bus     │                       │
│  │ (lifecycle)   │  │  │ (NATS)        │                       │
│  └───────┬───────┘  │  └───────┬───────┘                       │
│          │          │          │                               │
│          ▼          │          ▼                               │
│  ┌───────────────┐  │  ┌───────────────┐                       │
│  │ GraphExecutor │  │  │ Checkpoint    │                       │
│  │ (ri-agent-    │  │  │ Manager       │                       │
│  │  graph)       │  │  │ (SQLite)      │                       │
│  └───────────────┘  │  └───────────────┘                       │
└─────────────────────┴───────────────────────────────────────────┘
         │                      │
         │ NATS JetStream        │ Filesystem
         ▼                      ▼
  ┌──────────────┐      ┌──────────────┐
  │  NATS Server │      │  PostgreSQL  │
  │  (events)    │      │  (persistent)│
  └──────────────┘      └──────────────┘
```

### Interface Contract (proto/agent.proto)

```protobuf
syntax = "proto3";
package rapidagent.v1;

service AgentControl {
  // Spawn a new agent instance
  rpc Spawn(SpawnRequest) returns (SpawnResponse);
  
  // Stop an agent instance
  rpc Stop(StopRequest) returns (StopResponse);
  
  // Get agent status (streaming)
  rpc Status(StatusRequest) returns (stream StatusUpdate);
  
  // Get agent logs (streaming)
  rpc Logs(LogsRequest) returns (stream LogEntry);
  
  // Get agent metrics
  rpc Metrics(MetricsRequest) returns (MetricSnapshot);
}

message SpawnRequest {
  string tenant_id = 1;
  string agent_id = 2;
  bytes artifact = 3;  // Compiled .rag or .rapidagent
  map<string, string> config = 4;
}

message SpawnResponse {
  string instance_id = 1;
  uint64 started_at = 2;
}

message StatusRequest {
  string instance_id = 1;
}

message StatusUpdate {
  string instance_id = 1;
  string state = 2;  // running, paused, stopped, errored
  uint64 turn_count = 3;
  uint64 token_count = 4;
}

message LogEntry {
  string instance_id = 1;
  string level = 2;  // info, warn, error
  string message = 3;
  uint64 timestamp = 4;
  map<string, string> attributes = 5;
}

message MetricsRequest {
  string instance_id = 1;
}

message MetricSnapshot {
  string instance_id = 1;
  map<string, double> counters = 2;
  map<string, double> gauges = 3;
  map<string, double> histograms = 4;
}
```

### Implementation Strategy

```rust
// rapidagent-daemon/src/main.rs
use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize NATS connection
    let nats = NATS.connect("nats://localhost:4222").await?;
    
    // Initialize PostgreSQL connection
    let pg = PgPool::connect("postgres://...").await?;
    
    // Create agent manager
    let (tx, rx) = mpsc::channel::<AgentCommand>(1024);
    tokio::spawn(agent_manager(rx, nats.clone(), pg.clone()));
    
    // Start gRPC server
    let agent_control = AgentControlServiceImpl::new(tx.clone());
    
    Server::builder()
        .add_service(AgentControlServer::new(agent_control))
        .serve("0.0.0.0:50051".parse()?)
        .await?;
    
    Ok(())
}
```

---

## Security Considerations

### Authentication

1. **gRPC**: Use `tonic::async_trait` interceptors for JWT validation
2. **WebSocket**: Use query parameter tokens or upgrade handshake auth
3. **NATS**: Use JWT accounts with scoped permissions

### TLS

```rust
use tokio_rustls::rustls::{Certificate, PrivateKey};

let certs = rustls_pemfile::certs(&mut buf)?.into_iter().map(Certificate).collect();
let key = rustls_pemfile::private_key(&mut buf)?.expect("private key");

let config = rustls::ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(certs, key)?;

let tls = TlsAcceptor::from(Arc::new(config));
```

### Rate Limiting

Use Tower middleware for rate limiting:
```rust
use tower_limit::RateLimitLayer;

let rate_limited = ServiceBuilder::new()
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
    .service(agent_service);
```

---

## References

- [Bud Architecture Specification](https://github.com/magicloops/bud/blob/main/bud.spec.md)
- [Velocity-Dex GitHub](https://github.com/syafiqeil/Velocity-Dex)
- [Rust gRPC Authentication Service](https://dmitrii.app/rust-grpc-authentication-service-cross-platform-client/)
- [Tonic Documentation](https://docs.rs/tonic)

---

*Research completed: 2026-09-15*  
*Next steps: Create proto/agent.proto with this interface contract*
