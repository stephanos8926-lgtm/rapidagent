# ADR-002: Blocking Warden via NATS Request/Reply

---
title: "ADR-002: Blocking Warden via NATS Request/Reply"
status: Accepted
date: 2026-09-15
deciders: Lucien, Steven Page
consulted: Security Team
informed: All contributors

---

## Context

The Security Warden must enforce RBAC decisions synchronously before sensitive operations proceed. Asynchronous "fire and forget" patterns are unacceptable for security gates because:
1. Agents must not proceed until authorization is confirmed
2. Denial must be immediate to prevent damage
3. Audit trail must capture every decision

Existing patterns evaluated: REST API calls, shared database checks, in-process function calls, NATS request/reply.

## Decision

We will use **NATS request/reply pattern** for synchronous Warden authorization checks.

**Decision:** We will implement blocking request/reply where:
1. Engine publishes `CheckRequest` to `warden.{tenant}.requests`
2. Warden subscribes to that subject, evaluates policy
3. Warden replies to `warden.{tenant}.replies.{requestId}` with `CheckResponse`
4. Engine blocks (async/await) until response or timeout (5s)
5. Timeout or error → **fail-closed** (deny)

## Alternatives Considered

| Option | Description | Pros | Cons | Reason Rejected |
|--------|-------------|------|------|-----------------|
| **REST API** | HTTP POST to Warden endpoint | Simple, familiar | HTTP overhead, no native request/reply, harder to implement timeout | NATS is faster, native req/rep |
| **Shared Database** | Check permissions in PG | Durable, queryable | Latency, locking, complexity | Too slow for per-tool-call checks |
| **In-Process** | Call Warden function directly | Fastest, simplest | Tight coupling, hard to scale, hard to test | We need distributed architecture |
| **Async + Poll** | Submit request, poll for result | Decouples timing | Complex state management, latency | Blocking is simpler and safer |
| **NATS Request/Reply** | Native req/rep pattern | Fast, simple, built-in timeout | Requires NATS dependency | **Chosen** |

## Consequences

### Positive
- **Simplicity**: NATS handles request correlation, timeout, reply routing
- **Performance**: Sub-millisecond round-trip vs REST (no HTTP overhead)
- **Reliability**: NATS guarantees delivery, supports retries
- **Fail-closed**: Timeout → deny is explicit and safe
- **Observable**: Easy to measure Warden latency via NATS metrics

### Negative
- **NATS dependency**: All agents need NATS connectivity
- **Blocking**: Engine threads block on Warden (mitigated by async/await)
- **Single Warden**: Bottleneck if Warden is slow (mitigated by caching policies)
- **Debugging**: Must trace NATS request ID through system

### Neutral/Follow-ups
- Monitor Warden response latency (target: <100ms p99)
- Implement circuit breaker for Warden unhealthiness
- Cache frequently-check policies to reduce Warden load

## Implementation Notes

- Use `nats.request()` with 5s timeout
- Generate unique `request_id` per check (UUID v4)
- Reply subject: `warden.{tenant}.replies.{request_id}`
- Log all requests/responses with correlation ID
- Implement circuit breaker after 5 consecutive failures

## References
- [SPEC-002: Blocking Warden Request/Reply](../specs/SPEC-002-warden-blocking.md)
- [NATS Request/Reply Pattern](https://docs.nats.io/running-a-nats-service/services/msg_patterns)
- [Security Warden Architecture](../plans/architecture-component-list.md#tier-6-security-warden)
