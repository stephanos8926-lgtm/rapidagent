---
title: "ADR Template"
description: "Architecture Decision Record template using Nygard format"
category: template
tags:
  - adr
  - architecture
  - documentation
---

# ADR-[NNN]: [Short Title in Title Case]

**Status**: [Proposed | Accepted | Deprecated | Superseded by ADR-[NNN]]
**Date**: YYYY-MM-DD
**Deciders**: [Names/Roles who made the decision]
**Consulted**: [Names/Roles consulted]
**Informed**: [Who needs to know]

## Context

[What is the issue or opportunity? Describe the forces at play: technical constraints, business requirements, team capabilities, external dependencies. Be value-neutral and factual.]

**Forces:**
- [Force 1: e.g., "Need to support real-time updates"]
- [Force 2: e.g., "Current database doesn't support pub/sub"]
- [Force 3: e.g., "Team has Kafka experience"]

## Decision

[What is the decision? State in active voice: "We will..."]

**Decision:** We will [specific decision statement].

## Alternatives Considered

| Option | Description | Pros | Cons | Reason Rejected |
|--------|-------------|------|------|-----------------|
| **A** | [Option A] | [Pros] | [Cons] | [Why not chosen] |
| **B** | [Option B] | [Pros] | [Cons] | [Why not chosen] |
| **C** | Do nothing | [Status quo benefits] | [Status quo costs] | [Why change is needed] |

## Consequences

### Positive
- [Benefit 1: What becomes easier?]
- [Benefit 2: Cost savings, performance gain, etc.]
- [Benefit 3: Team productivity improvement]

### Negative
- [Trade-off 1: What becomes harder?]
- [Trade-off 2: Additional complexity, learning curve]
- [Trade-off 3: Dependency on external service]

### Neutral/Follow-ups
- [Monitoring required]
- [Future reconsideration triggers]
- [Metrics to track]

## Implementation Notes

[Optional: Link to implementation details, PRs, or related ADRs]

## References
- [Related ADRs]
- [External documentation]
- [Research findings]
