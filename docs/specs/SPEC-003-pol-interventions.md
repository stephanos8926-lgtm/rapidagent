# SPEC-003: POL Control Plane Intervention Management

---
name: POL-Intervention-Management
description: POL Control Plane intervention creation, tracking, resolution, and escalation
status: draft
created: 2026-09-15
author: Lucien
related-adrs:
  - "ADR-003"
---

# Spec: POL Control Plane Intervention Management

## Purpose
Define the intervention lifecycle managed by the POL Control Plane. Enables autonomous failure detection, corrective action, and human escalation.

## Requirements

### R1: Intervention Creation
The system SHALL create interventions when:

1. POL Subscriber detects `worker.failed`, `tool.denied`, `task.failed`, or `violation` events
2. Policy Evaluator detects policy violation
3. Admin manually creates intervention via gRPC/WebSocket API
4. Warden reports critical security decision

Intervention schema:
```json
{
  "id": "intv-{uuid}",
  "task_id": "string",
  "type": "system_intervention",
  "source": "POL",
  "priority": "critical|high|medium|low",
  "reason": "string",
  "guidance": "string",
  "status": "pending|active|resolved|escalated|cancelled",
  "action": "cancel|retry|escalate|null",
  "payload": {},
  "created_at": "ISO8601",
  "resolved_at": "ISO8601|null"
}
```

#### Scenario: Tool Failure Detection
- **GIVEN** worker fails tool call 3 times consecutively
- **WHEN** POL Subscriber detects `tool.failed` event
- **THEN** system SHALL create intervention with `priority: "high"`, `reason: "repeated_tool_failure"`

#### Scenario: Policy Violation
- **GIVEN** agent attempts to access disallowed endpoint
- **WHEN** Policy Evaluator returns `allowed: false`
- **THEN** system SHALL create intervention with `priority: "critical"`, `reason: "policy_violation"`

### R2: Intervention Resolution
The system SHALL support three resolution actions:

| Action | Effect | Use Case |
|--------|--------|----------|
| `cancel` | Mark task as FAILED | Unrecoverable failure |
| `retry` | Reset task to EXECUTING | Transient failure |
| `escalate` | Notify admin, wait for approval | Unknown failure |

#### Scenario: Retry on Transient Failure
- **GIVEN** intervention created for `tool.denied` with reason "rate_limit"
- **WHEN** admin resolves with `action: "retry"`
- **THEN** task state transitions back to EXECUTING

#### Scenario: Cancel on Critical Failure
- **GIVEN** intervention created for `task.failed` with reason "security_violation"
- **WHEN** admin resolves with `action: "cancel"`
- **THEN** task state transitions to FAILED, intervention marked resolved

### R3: Human Escalation
The system SHALL escalate to human when:

1. Intervention priority is `critical` and auto-resolution fails
2. Admin explicitly requests escalation
3. Policy violation involves sensitive operations (delete, exec)

Escalation flow:
1. Log escalation event
2. Send notification via Telegram/Discord webhook
3. Wait for admin approval (timeout: 5 minutes)
4. On approval: resolve with `action: "approve"`
5. On rejection: resolve with `action: "deny"`
6. On timeout: resolve with `action: "deny"` (fail-closed)

#### Scenario: Escalation Timeout
- **GIVEN** critical intervention created at 14:00:00
- **WHEN** no admin response by 14:05:00
- **THEN** system SHALL auto-deny and log `ESCALATION_TIMEOUT`

### R4: Persistence
The system SHALL persist interventions using a **dual-tier storage strategy**:

#### Primary: PostgreSQL
- Transactional integrity for intervention CRUD
- Replication for high availability
- Row-level security for multi-tenancy
- Column encryption for confidential data

#### Secondary: Local JSONL Journal
- Append-only local journal for crash recovery
- Sync to PostgreSQL when available
- Survives PostgreSQL outages

Local journal schema (append-only, encrypted):
```jsonl
{"id": "intv-abc", "created_at": "...", "status": "pending", "priority": "critical", ...}
{"id": "intv-abc", "created_at": "...", "status": "resolved", "resolved_at": "...", "action": "retry"}
```

#### Encryption at Rest
- Local journal SHALL be encrypted with `age` using agent-specific keys
- Keys stored in Hashicorp Vault (never on disk in plaintext)
- PostgreSQL uses AES-256-GCM column encryption

#### Sync Strategy
```
Local Journal ──[background sync]──► PostgreSQL
     ▲                                  │
     │                                  ▼
     └────[on startup recovery]─────────┘
```

#### Scenario: PG Outage
- **GIVEN** PostgreSQL is unavailable
- **WHEN** intervention is created
- **THEN** system SHALL write to local JSON journal
- **WHEN** PG becomes available
- **THEN** system SHALL sync pending interventions to PG

### R5: WebSocket Streaming
The system SHALL broadcast intervention updates to WebSocket clients:

- Connect to `/ws/interventions`
- Receive real-time updates for all interventions (tenant-scoped)
- Support filtering by priority, status
- All WebSocket traffic encrypted via TLS

### R6: Data Classification
Interventions SHALL be classified by sensitivity:

| Priority | Sensitivity | Retention | Access |
|----------|-------------|-----------|--------|
| `critical` | confidential | 7 years | Admin + audit |
| `high` | confidential | 1 year | Admin + POL |
| `medium` | internal | 90 days | Admin |
| `low` | internal | 30 days | Admin |

### R7: Immutable Audit Trail
Resolved interventions SHALL form a cryptographically chained audit trail:
- Each resolution creates a new journal entry
- Entries include `prev_hash` for tamper detection
- Resolved interventions are append-only (no updates or deletes)
- Only soft-delete allowed for compliance requests (logged separately)

#### Scenario: Real-time Monitoring
- **GIVEN** admin opens WebSocket connection
- **WHEN** new intervention is created
- **THEN** admin receives instant WebSocket message with intervention details

## Non-Requirements
- The system SHALL NOT auto-resolve `critical` priority interventions without human approval
- The system SHALL NOT delete interventions (soft-delete only for compliance)
- The system SHALL NOT modify resolved interventions (immutable audit trail)

## Design Notes

### Architecture
```
POL Subscriber ──► POLControlPlane ──► Intervention Store (PG + JSON)
                                        │
                                        ├─► WebSocket Stream (real-time)
                                        ├─► gRPC Status (query)
                                        └─► Human Escalation (Telegram/Discord)
```

### Risk and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Intervention loss during PG outage | Medium | High | Local JSON journal with sync |
| Escalation notification delivery failure | Low | Medium | Retry with exponential backoff |
| WebSocket connection drop | Medium | Low | Auto-reconnect + state re-sync |
| Duplicate intervention creation | Low | Medium | Idempotency key on creation |

## Open Questions
- Should we support intervention chaining (one intervention triggers another)?
- Should admin approval require multi-factor authentication?
- How do we handle intervention backlog during extended outages?

## References
- [POL Control Plane Component](../plans/architecture-component-list.md#tier-5-pol-control-plane)
- [NATS JetStream for Durability](https://docs.nats.io/nats-concepts/jetstream)
- [WebSocket Protocol](https://tools.ietf.org/html/rfc6455)
