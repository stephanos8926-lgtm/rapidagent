---
name: [FEATURE-NAME]
description: [One-line description of the feature]
status: draft | proposed | accepted | implemented
created: YYYY-MM-DD
author: [Author name]
related-adrs: [ADR numbers if applicable]
---

# Spec: [Feature Name]

## Context

[What problem are we solving? What is the current state and why does it need to change? Be specific about user needs and business drivers.]

## Requirements

### R1: [Requirement Name]
[Describe what the system SHALL do. Use SHALL/MUST for normative requirements.]

#### Scenario: [Scenario name]
- **GIVEN** [precondition or initial state]
- **WHEN** [action or event occurs]
- **THEN** [expected outcome]

### R2: [Requirement Name]
[Description]

#### Scenario: [Scenario name]
- **GIVEN** [precondition]
- **WHEN** [action]
- **THEN** [expected outcome]

#### Scenario: [Edge case name]
- **GIVEN** [edge case precondition]
- **WHEN** [edge case action]
- **THEN** [expected outcome]

## Non-Requirements
[What this feature explicitly does NOT do]

- [Non-requirement 1]
- [Non-requirement 2]

## Design Notes
[Implementation details, architecture decisions, technical constraints. Keep separate from behavior specs.]

### Architecture
[High-level architecture description]

### Data Model
[Key data structures and relationships]

### API Contract
[External interfaces and contracts]

### Risks and Mitigations
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| [Risk description] | Medium | High | [Mitigation strategy] |

## Open Questions
[Questions that need resolution before implementation can begin]

- [Question 1]
- [Question 2]

## References
- [Related documentation or ADRs]
- [External resources]
