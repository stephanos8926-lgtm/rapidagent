# ADR-003: POL Control Plane as Daemon Service

---
title: "ADR-003: POL Control Plane as Daemon Service"
status: Accepted
date: 2026-09-15
deciders: Lucien, Steven Page
consulted: Architecture Team
informed: All contributors

---

## Context

The Platform Orchestration Layer (POL) must operate continuously, monitoring all agent activity and intervening when necessary. Two architectural options exist:
1. **Daemon Service**: Separate long-running process hosting POL logic
2. **Embedded Library**: POL logic embedded in each agent engine instance

The decision impacts: resource usage, failure isolation, scalability, and update strategy.

## Decision

We will implement POL as a **daemon service** running on the `infra` VM, not embedded in agent engines.

**Decision:** We will deploy `POLControlPlane` as a systemd service (`rapidagent-pol.service`) on `infra` VM that:
1. Subscribes to `pol.{tenant}.events` NATS stream
2. Maintains intervention state in PostgreSQL
3. Exposes gRPC API for agent control
4. Broadcasts updates via WebSocket to admins
5. Runs independently of agent engines (failure isolation)

## Alternatives Considered

| Option | Description | Pros | Cons | Reason Rejected |
|--------|-------------|------|------|-----------------|
| **Daemon Service** | Separate process on infra VM | Failure isolation, centralized, scalable | Additional process to manage, latency for cross-VM calls | **Chosen** - better for enterprise |
| **Embedded Library** | POL logic in each engine | Lower latency, no network dependency | Each engine duplicates POL, hard to update centrally, state management complexity | Scaling issue: 100 agents = 100 POL instances |
| **Serverless Function** | Cloud function triggered by events | Auto-scaling, pay-per-use | Cold starts, vendor lock-in, complexity | Overkill for internal enterprise use |
| **Kubernetes Operator** | K8s controller pattern | Declarative, self-healing | Requires K8s, over-engineered for our scale | We use systemd + Incus, not K8s |

## Consequences

### Positive
- **Centralized**: Single POL instance manages all agents
- **Isolated**: POL failure doesn't crash agent engines
- **Scalable**: Can add more POL instances for different tenants
- **Observable**: Centralized logs, metrics, intervention history
- **Updatable**: Update POL without restarting agents

### Negative
- **Latency**: Cross-VM calls add ~1-5ms (acceptable for POL checks)
- **Complexity**: Additional service to deploy and monitor
- **Single point of failure**: POL downtime = no interventions (mitigated by agent self-recovery)
- **Resource usage**: POL daemon uses ~50MB RAM, negligible

### Neutral/Follow-ups
- Plan POL HA cluster for Horizon 3 (multiple POL instances)
- Implement POL health probe for systemd restart on failure
- Document POL scaling guidelines in AGENTS.md

## Implementation Notes

- Deploy as `rapidagent-pol.service` on `infra` VM
- Config: `/etc/rapidagent/pol.yaml`
- Logs: `/var/log/rapidagent/pol.log`
- Data: PostgreSQL `interventions` table
- Health: HTTP `/health` endpoint on port 8081
- Restart policy: `Restart=always`, `RestartSec=5`

## References
- [SPEC-003: POL Intervention Management](../specs/SPEC-003-pol-interventions.md)
- [POL Control Plane Component](../plans/architecture-component-list.md#tier-5-pol-control-plane)
- [Daemon Deployment Strategy](../plans/RapidAgent Framework Architecture and Design Vision Document.md#infrastructure-topology-production)
