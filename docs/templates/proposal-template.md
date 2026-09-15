---
title: "Change Proposal Template"
description: "OpenSpec change proposal template"
category: template
tags:
  - openspec
  - proposal
  - change-management
---

# Change Proposal: [Feature/Change Name]

## Metadata

- **Change ID**: [CHANGE-001]
- **Status**: [Draft | Review | Approved | Implemented | Archived]
- **Created**: YYYY-MM-DD
- **Author**: [Name]
- **Related Specs**: [spec-path/]
- **Related ADRs**: [ADR numbers]

## Summary

[One-paragraph summary of the change and its motivation]

## Motivation

[Why is this change needed? What problem does it solve?]

### Problem Statement
[Clear description of the current problem]

### Current Behavior
[How does the system currently behave?]

### Desired Behavior
[How should the system behave after this change?]

## Scope

### In Scope
- [What will be changed]
- [What will be affected]

### Out of Scope
- [What will NOT be changed]
- [What is explicitly excluded]

## Requirements

### New Requirements
| ID | Requirement | Source | Priority |
|----|-------------|--------|----------|
| REQ-001 | [Requirement text] | [Source] | High |
| REQ-002 | [Requirement text] | [Source] | Medium |

### Modified Requirements
| ID | Current | Proposed | Rationale |
|----|---------|----------|-----------|
| REQ-010 | [Current text] | [New text] | [Why changing] |

### Removed Requirements
| ID | Requirement | Rationale | Migration |
|----|-------------|-----------|-----------|
| REQ-020 | [Old text] | [Why removing] | [How to migrate] |

## Scenarios

### Scenario: [Scenario name]
- **GIVEN** [precondition]
- **WHEN** [action]
- **THEN** [expected outcome]

### Scenario: [Edge case name]
- **GIVEN** [edge case precondition]
- **WHEN** [edge case action]
- **THEN** [expected outcome]

## Design Approach

[High-level approach to implementation. Keep this separate from detailed design.]

### Architecture Changes
[What architectural changes are needed?]

### Data Model Changes
[Any database or data structure changes]

### API Changes
[Any external interface changes]

## Migration Plan

### Backward Compatibility
[How is backward compatibility maintained?]

### Upgrade Steps
1. [Step 1]
2. [Step 2]
3. [Step 3]

### Rollback Plan
[How to revert if needed]

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| [Risk 1] | Medium | High | [Mitigation] |
| [Risk 2] | Low | Medium | [Mitigation] |

## Open Questions
- [Question 1]
- [Question 2]

## References
- [Related documentation]
- [External resources]
