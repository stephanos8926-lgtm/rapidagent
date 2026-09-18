# ADR-008: Policy Engine Choice

---
title: "ADR-008: Policy Engine Choice"
status: Accepted
date: 2026-09-16
deciders: Lucien, Steven Page
consulted: RapidWebs Security Team
informed: All contributors

---

## Context

The RapidAgent Framework requires policy evaluation for:
- Agent spawn/stop authorization
- Tool access control
- Data quota enforcement
- Intervention approval

Options evaluated: embedded OPA/Rego, custom RBAC,OPA server, CASL.

## Decision

We will use **embedded OPA/Rego** for policy evaluation.

**Decision:** Integrate `opa-wasm` crate for embedded policy evaluation with:
- Bundle-based policy distribution
- <1ms p99 latency
- Built-in unit testing
- Hot-reload support

## Alternatives Considered

| Option | Pros | Cons | Reason Rejected |
|--------|------|------|-----------------|
| **Custom RBAC** | Simple, no dependencies | Hard to express complex policies, reinventing wheel | Too limiting for our use case |
| **OPA server** | Separation of concerns, standalone | Network latency, operational overhead | Embedded is faster, simpler |
| **CASL** | JavaScript-based | Language mismatch, smaller ecosystem | Rego is better suited for declarative policies |
| **OPA embedded** ✅ | Fast, declarative, testable, mature | Learning curve for Rego | Best fit for our requirements |

## Consequences

### Positive
- **Performance**: <1ms p99 for simple policies
- **Declarative**: Rego is expressive and readable
- **Testable**: Built-in unit testing framework
- **Flexible**: Complex policies easy to express
- **Mature**: OPA is widely adopted

### Negative
- **Learning curve**: Team needs Rego training
- **Debugging**: Policy evaluation can be hard to debug
- **Versioning**: Policy updates require care
- **Bundle management**: Need distribution mechanism

### Mitigations
- Document Rego patterns in AGENTS.md
- Implement policy testing in CI
- Use Git for policy version control
- Add policy evaluation logging

## Implementation Notes

- Use `opa-wasm` crate v0.15+
- Store policies in `docs/policies/` directory
- Implement bundle endpoint for policy distribution
- Add policy unit tests alongside Rego files
- Log all policy evaluations for audit

## References
- [SPEC-007: OPA Policy Engine](../specs/SPEC-007-opa-policy.md)
- [OPA Documentation](https://www.openpolicyagent.org/docs/)
