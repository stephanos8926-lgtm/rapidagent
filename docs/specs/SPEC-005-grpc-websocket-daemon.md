# SPEC-005: gRPC + WebSocket Daemon Architecture

---
name: gRPC-WebSocket-Daemon
description: Dual-interface daemon for agent control (gRPC) and real-time updates (WebSocket)
status: draft
created: 2026-09-16
author: Lucien
related-adrs:
  - "ADR-007"
---

# Spec: gRPC + WebSocket Daemon

## Goal
Implement a dual-interface daemon providing typed control plane via gRPC and real-time event streaming via WebSocket.

## Context
Phase 2 research established the dual-interface pattern proven by Bud and Velocity-Dex:
- gRPC for typed control operations (spawn, stop, status, config)
- WebSocket for real-time event streaming (logs, metrics, state changes)
- Protocol multiplexing reduces operational complexity

## Requirements

### R1: gRPC Service Definition

```protobuf
syntax = "proto3";
package rapidagent.v1;

service AgentControl {
  // Spawn a new agent execution
  rpc Spawn(SpawnRequest) returns (SpawnResponse);
  // Stop an running agent
  rpc Stop(StopRequest) returns (StopResponse);
  // Get agent status
  rpc Status(StatusRequest) returns (StatusResponse);
  // List running agents
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
  // Pause/resume agent
  rpc Pause(PauseRequest) returns (PauseResponse);
  // Get execution history
  rpc GetHistory(GetHistoryRequest) returns (GetHistoryResponse);
}

message SpawnRequest {
  string agent_id = 1;
  string template = 2;
  map<string, string> inputs = 3;
  string workspace_id = 4;
  int32 priority = 5;
}

message SpawnResponse {
  string execution_id = 1;
  string status = 2;
  int64 started_at = 3;
}

message StopRequest {
  string execution_id = 1;
  string reason = 2;
}

message StatusRequest {
  string execution_id = 1;
}

message StatusResponse {
  string execution_id = 1;
  string status = 2;
  string current_node = 3;
  int64 started_at = 4;
  int64 completed_at = 5;
  repeated ExecutionStep steps = 6;
}

message ExecutionStep {
  string node_id = 1;
  string status = 2;
  int64 started_at = 3;
  int64 completed_at = 4;
  string receipt_hash = 5;
}
```

### R2: WebSocket Event Streams

| Stream | Event Type | Use Case |
|--------|------------|----------|
| `agents.{id}.logs` | LogEvent | Real-time logging |
| `agents.{id}.metrics` | MetricEvent | Performance metrics |
| `agents.{id}.state` | StateEvent | Node state changes |
| `agents.{id}.receipts` | ReceiptEvent | Execution receipts |
| `agents.{id}.errors` | ErrorEvent | Error notifications |

**WebSocket Message Format:**
```json
{
  "type": "log|metric|state|receipt|error",
  "execution_id": "uuid",
  "timestamp": 1234567890,
  "data": {}
}
```

### R3: Authentication
- gRPC: mTLS + JWT token in metadata
- WebSocket: JWT in URL query parameter (handshake)
- Subject-based ACL: `agents.{tenant}.{id}.>`

### R4: Connection Management
- gRPC: Unary + streaming responses
- WebSocket: Auto-reconnect with exponential backoff
- Heartbeat: Ping/Pong every 30s
- Max connection per agent: 10

### R5: Testing Requirements
- [ ] gRPC service unit tests
- [ ] WebSocket connection tests
- [ ] Authentication tests
- [ ] Load test: 100 concurrent agents
- [ ] Reconnection test

## Out of Scope
- REST API (future gateway layer)
- GraphQL support
- WebSocket compression

## References
- [Phase 2 Research: gRPC + WebSocket Daemons](../research/grpc-websocket-daemon.md)
- [SPEC-001: EHP Envelope](./SPEC-001-ehp-envelope.md)
