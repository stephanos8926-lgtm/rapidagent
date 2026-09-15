# SPEC-002: Blocking Warden Request/Reply

---
name: Warden-Blocking-Request
description: Security Warden blocking synchronization protocol via NATS request/reply
status: draft
created: 2026-09-15
author: Lucien
related-adrs:
  - "ADR-002"
---

# Spec: Blocking Warden Request/Reply

## Purpose
Define the synchronous blocking protocol for Security Warden decisions. Ensures all sensitive operations (filesystem, tool execution, network) are explicitly authorized before proceeding.

## Requirements

### R1: Blocking Request Pattern
The system SHALL implement a request/reply pattern where:

1. Engine submits a `CheckRequest` envelope to `warden.{tenant}.requests`
2. Warden processes the request synchronously
3. Warden responds to `warden.{tenant}.replies.{requestId}` with `CheckResponse`
4. Engine blocks until response or timeout

#### Scenario: Tool Execution Check
- **GIVEN** agent calls `shell_exec("rm -rf /")`
- **WHEN** engine submits CheckRequest with `tool_name: "shell_exec"`, `args: ["rm", "-rf", "/"]`
- **THEN** Warden SHALL evaluate policy and respond with `allowed: false`, `reason: "Path / is outside workspace"`

#### Scenario: Filesystem Write Check
- **GIVEN** agent writes to `/home/sysop/.forge/config.yaml`
- **WHEN** engine submits CheckRequest with `operation: "fs.write"`, `path: "/home/sysop/.forge/config.yaml"`
- **THEN** Warden SHALL check principal permissions and respond with `allowed: true` or `allowed: false`

### R2: Timeout and Fail-Closed
The system SHALL implement strict timeout semantics:

| Parameter | Default | Configurable |
|-----------|---------|--------------|
| Request timeout | 5000ms | YES |
| Grace period (first boot) | 30s | YES |
| Fail-closed on timeout | YES | NO |
| Fail-closed on error | YES | NO |

#### Scenario: Warden Unavailable
- **GIVEN** Warden service is down
- **WHEN** engine submits CheckRequest
- **THEN** after 5000ms timeout, engine SHALL deny the operation and log `WARDEN_TIMEOUT`

#### Scenario: First Boot Grace Period
- **GIVEN** daemon starts and Warden is initializing
- **WHEN** engine submits CheckRequest within first 30 seconds
- **THEN** engine SHALL allow the operation (grace period) and log `WARDEN_GRACE_PERIOD`

### R3: Request/Response Schema
The system SHALL define explicit request/response types:

```json
// CheckRequest
{
  "request_id": "uuid",
  "principal_type": "USER|AGENT|SYSTEM|ADMIN",
  "principal_id": "string",
  "operation": "fs.read|fs.write|fs.delete|tool.execute|network.request|memory.delete",
  "resource": "string",
  "context": {
    "session_id": "string",
    "workspace_id": "string",
    "trace_id": "string"
  },
  "metadata": {}
}

// CheckResponse
{
  "request_id": "uuid",
  "allowed": boolean,
  "reason": "string",
  "metadata": {}
}
```

### R4: Circuit Breaker
The system SHALL implement a circuit breaker to prevent Warden unhealthiness from blocking all operations:

- **Closed state**: Normal operation, requests go to Warden
- **Open state**: Warden unresponsive, all requests fail-closed
- **Half-open state**: After N seconds, allow one test request through

Thresholds:
- `failure_threshold`: 5 consecutive failures
- `recovery_timeout`: 30 seconds
- `test_request_timeout`: 2 seconds

#### Scenario: Circuit Breaker Trips
- **GIVEN** Warden has failed 5 consecutive requests
- **WHEN** engine submits CheckRequest
- **THEN** circuit breaker SHALL be open, request FAILS IMMEDIATELY with `CIRCUIT_OPEN`

#### Scenario: Circuit Breaker Recovery
- **GIVEN** circuit breaker is open
- **WHEN** 30 seconds elapse
- **THEN** circuit breaker enters half-open state, allows one test request
- **IF** test request succeeds → CLOSED
- **IF** test request fails → stays OPEN

## Non-Requirements
- The protocol SHALL NOT support async/non-blocking checks (by design)
- The protocol SHALL NOT cache Warden decisions (each check is evaluated fresh)
- The protocol SHALL NOT retry on Warden timeout (fail-closed)

## Design Notes

### Architecture
```
Engine ──CheckRequest──► NATS ──► Warden
                              ▲
Engine ◄──CheckResponse──────┘
     │
     ├─[timeout]─► Deny + Log
     └─[circuit_open]─► Deny + Log
```

### Risk and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Warden deadlock | Medium | Critical | Circuit breaker + timeout |
| First-boot denial | Low | High | Grace period configuration |
| Replay attacks | Low | Medium | Request ID uniqueness + TTL |
| Performance overhead | Medium | Medium | Benchmark <100ms p99 |

## Open Questions
- Should we support priority-based request preemption?
- Should Warden support batch checks for efficiency?
- How do we handle Warden scaling (multiple instances)?

## References
- [Security Warden Component](../plans/architecture-component-list.md#tier-6-security-warden)
- [NATS Request/Reply Pattern](https://docs.nats.io/running-a-nats-service/services/msg_patterns)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
