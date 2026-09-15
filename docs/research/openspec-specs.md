# OpenSpec Spec Writing Guide

## Overview
OpenSpec is a spec-driven development methodology that separates WHAT the system does from HOW it's built. Specs describe behavior using requirements and scenarios—concrete examples that prove those requirements.

## Core Principles

### 1. Behavior Over Implementation
A spec says what your system does, in terms anyone could check—not how it's built.
- ✅ "The system SHALL return a 401 status when the token is expired"
- ❌ "The system SHALL check the token expiration field in the database"

### 2. One Requirement, One Statement
If a requirement has three "and also" clauses, it's really three requirements. Split them.

### 3. Observable Behavior
Someone outside the code should be able to tell whether it holds.
- Observable: "The system SHALL show an error banner when the upload exceeds 10 MB"
- Not observable: "The system SHALL handle large uploads gracefully"

### 4. RFC 2119 Keywords
| Keyword | Meaning |
|---------|---------|
| `MUST` / `SHALL` | A hard requirement. Non-negotiable. |
| `SHOULD` | A strong recommendation, with room for a justified exception. |
| `MAY` | Genuinely optional. |

Reach for `MUST`/`SHALL` by default.

## Spec Structure

```markdown
## Purpose
<!-- New capabilities only: one or two sentences (50+ characters) -->
[What this capability is for]

## Requirements

### Requirement: [Name]
[Behavior statement using SHALL/MUST]

#### Scenario: [Scenario name]
- **GIVEN** [precondition]
- **WHEN** [action/condition]
- **THEN** [expected outcome]
```

## Delta Operations

When modifying existing specs, use these sections:

### `## ADDED Requirements`
Brand-new behavior that didn't exist before.

### `## MODIFIED Requirements`
Behavior that already existed and is changing. Include the full new version.

### `## REMOVED Requirements`
Behavior going away. Must include **Reason** and **Migration**.

### `## RENAMED Requirements`
Name changes only. Use FROM:/TO: format.

## Common Pitfalls

### Too Big
A good change has one intent you can say in a sentence.
- ✅ "Add a dark-mode toggle"
- ❌ "Add a dark-mode toggle, fix the theme API, and update the settings page"

### Implementation Leakage
Don't bake implementation details into requirements.
- ❌ "The system SHALL use Redis for session storage"
- ✅ "The system SHALL persist sessions across process restarts"

### Missing Scenarios
Every requirement MUST have at least one scenario that exercises it.

## When to Use OpenSpec

Use OpenSpec for:
- Cross-team or cross-repo changes
- API/contract changes
- Security/privacy concerns
- Changes where ambiguity is likely to cause expensive rework

For trivial changes, skip OpenSpec or use `skip_specs: true`.

## References
- [OpenSpec Documentation](https://openspec.dev/docs)
- [Writing Good Specs](https://openspec.dev/docs/writing-specs)
- [Core Concepts](https://openspec.dev/docs/core-concepts)
