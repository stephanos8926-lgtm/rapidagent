# SPEC-001: EHP Envelope v1 Schema

---
name: EHP-Envelope-v1
description: Canonical event envelope schema for the Event Horizon Pipeline
status: draft
created: 2026-09-15
author: Lucien
related-adrs:
  - "ADR-001"
---

# Spec: EHP Envelope v1 Schema

## Purpose
Define the canonical message format for all inter-component communication in the RapidAgent Framework. Provides schema consistency across Rust, Python, and TypeScript implementations.

## Requirements

### R1: Schema Definition
The system SHALL define a complete EHP Envelope schema with the following fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string (UUID v4) | YES | Unique message identifier |
| `type` | enum | YES | Message type: `COMMAND`, `EVENT`, `TELEMETRY`, `HEARTBEAT`, `ALERT` |
| `timestamp` | number (Unix epoch ms) | YES | Message creation time |
| `priority` | number (0-255) | YES | Priority level (255 = highest) |
| `source` | object | YES | Message origin: `{id, type, version?}` |
| `destination` | string | YES | Target subject/topic |
| `topic` | string | YES | Semantic topic identifier |
| `payload` | any | YES | Message-specific data |
| `context` | object | YES | Operational context: `{requestId, workspaceId, sessionId?, operationalContext?}` |
| `auth` | object | YES | Authentication: `{uid, role, permissions[], emailVerified?}` |
| `signature` | string | NO | Ed25519 signature (for COMMAND type) |

#### Scenario: Basic Event
- **GIVEN** an agent executes a tool call
- **WHEN** the tool completes
- **THEN** the system SHALL emit an EHP envelope with `type: "EVENT"`, `topic: "tool.execute"`, and payload containing tool name, arguments, and result

#### Scenario: Blocking Command
- **GIVEN** an agent needs to write a file
- **WHEN** the agent submits a COMMAND envelope to `warden.fs.write`
- **THEN** the Warden SHALL respond with ALLOW or DENY via the same envelope ID

### R2: Code Generation
The system SHALL generate type definitions from a single source of truth:

| Target | Format | File |
|--------|--------|------|
| Rust | `#[derive(Serialize, Deserialize)]` structs | `crates/rapidagent-envelope/src/schema.rs` |
| Python | Pydantic models | `src/rapidagent/envelope/models.py` |
| TypeScript | Zod schemas | `packages/envelope/src/schema.ts` |
| JSON Schema | OpenAPI-compatible | `schemas/ehp-envelope-v1.json` |
| Protobuf | Message definitions | `protos/ehp-envelope-v1.proto` |

#### Scenario: Cross-Language Consistency
- **GIVEN** the JSON Schema is the source of truth
- **WHEN** a developer runs `make generate-schemas`
- **THEN** all language bindings SHALL be regenerated and pass consistency checks

### R3: Backward Compatibility
The system SHALL enforce backward compatibility for envelope schema changes:

| Change Type | Allowed? | Migration Required |
|-------------|----------|-------------------|
| Add optional field | YES | NO |
| Add required field | NO | YES (schema version bump) |
| Change field type | NO | YES (deprecation period) |
| Remove field | YES (with deprecation) | NO |

#### Scenario: Schema Evolution
- **GIVEN** envelope version v1 is in production
- **WHEN** a developer adds an optional field `correlation_id`
- **THEN** older clients SHALL ignore the field and continue operating normally

### R4: Validation
The system SHALL validate all envelopes against the schema before processing:

- Invalid envelopes SHALL be rejected with a `SCHEMA_VALIDATION_ERROR` event
- Validation failures SHALL be logged with the offending envelope ID
- Validation SHALL occur at message boundaries (publish/subscribe)

#### Scenario: Malformed Envelope
- **GIVEN** a component publishes an envelope missing the `auth` field
- **WHEN** the subscriber receives the message
- **THEN** the system SHALL reject it and emit a validation error event

## Non-Requirements
- The schema SHALL NOT include message encryption (handled by NATS TLS)
- The schema SHALL NOT include compression (handled by transport)
- The schema SHALL NOT define business logic (only structure)

## Design Notes

### Architecture
```
schemas/ehp-envelope-v1.json
    │
    ├─→ Rust: cargo expand → schema.rs
    ├─→ Python: datamodel-code-generator → models.py
    ├─→ TypeScript: openapi-typescript → schema.ts
    └─→ Protobuf: protoreflect → proto file
```

### Risk and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Schema drift between languages | Medium | High | CI job validates all generated schemas match JSON source |
| Breaking change in production | Low | Critical | Schema versioning + deprecation period |
| Validation overhead | Low | Low | Validation only at message boundaries, not in hot path |

## Open Questions
- Should `signature` field support multiple algorithms (Ed25519, ECDSA)?
- Should we add a `ttl` field for message expiration?
- How do we handle schema migrations for running systems?

## References
- [EHP Envelope v1](../plans/RapidAgent Framework Architecture and Design Vision Document.md#1-event-horizon-pipeline-ehp)
- [NATS JetStream Documentation](https://docs.nats.io/)
- [OpenAPI Specification](https://spec.openapis.org/)
