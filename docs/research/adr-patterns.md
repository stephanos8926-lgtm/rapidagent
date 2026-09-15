# Architecture Decision Record (ADR) Patterns

## What is an ADR?
An Architecture Decision Record captures a significant architectural decision, the context that drove it, and the trade-offs considered. ADRs preserve the "why" that code cannot express.

## Two Primary Formats

### Nygard Format (Recommended for most teams)
Minimal, fast to write. Four core sections plus two extensions.

```markdown
# ADR-[NNN]: [Short Title]

**Status**: [Proposed | Accepted | Deprecated | Superseded by ADR-[NNN]]
**Date**: YYYY-MM-DD
**Owners**: [Name/Team]

## Context

[What is the issue? What forces are at play? State in value-neutral language.]

## Decision

[What is the decision? State in full sentences, active voice: "We will..."]

## Consequences

### Positive
- [Benefit]

### Negative
- [Trade-off or cost]

### Neutral/Follow-ups
- [What becomes easier/harder]

## Alternatives Considered
- **Option A**: [Description] — Chosen above.
- **Option B**: [Description] — Rejected because [reason].
- **Option C**: [Description] — Rejected because [reason].
- **Option D (do nothing)**: Rejected because [reason].

## Stakeholders
- **Deciders**: [Names]
- **Consulted**: [Names]
- **Informed**: [Channel/Team]
```

### MADR Format (More structured)
Use when teams need explicit decision drivers and option-level pros/cons.

```markdown
# ADR-[NNN]: [Short Title]

## Status
[Proposed | Accepted | Deprecated | Superseded]

## Context
[Problem statement]

## Drivers
- [Decision driver 1]
- [Decision driver 2]

## Options
- **Option A**: [Description]
  - Pros: [...]
  - Cons: [...]
- **Option B**: [Description]
  - Pros: [...]
  - Cons: [...]

## Decision
[Chosen option and rationale]

## Consequences
[What becomes easier/harder]
```

## When to Write an ADR

Write an ADR when a decision:
- Affects the structure or non-functional characteristics of the system
- Impacts dependencies, interfaces, or construction techniques
- Locks in a vendor, framework, or protocol that would be expensive to change later
- Encodes a trade-off the team has explicitly accepted

## Status Lifecycle

| Status | Meaning |
|--------|---------|
| `Proposed` | Decision is under discussion, open for comment |
| `Accepted` | Decision has been made and is current |
| `Deprecated` | Decision no longer applies but preserved for history |
| `Superseded by ADR-NNN` | Replaced by a later decision |

**Rule**: When superseding, mark old ADR with `Supersedes ADR-[NNN]` and new ADR with `Superseded by ADR-[NNN]`.

## Common Anti-Patterns

### The BDUF ADR
Writing a 20-page big-design-up-front document. ADRs should be 1-2 pages.

### The Retroactive ADR
Shipping the change first, writing the ADR three months later. Context is forgotten.

### The ADR for Everything
Not every PR needs an ADR. Reserve ADRs for architecturally significant decisions.

### The Update Trap
Updating an accepted ADR instead of creating a new superseding ADR. This loses history.

## Tooling

- **adr-tools**: CLI for managing ADR repositories
- **log4brains**: Visualizes ADRs as a knowledge graph
- **adr-viewer**: Renders ADRs as a website

## References
- [Documenting Architecture Decisions - Michael Nygard](http://www.nygard.com/blog/2011/11/documenting-architecture-decisions.html)
- [adr.github.io](https://adr.github.io/)
- [MADR Specification](https://adr.github.io/madr/)
