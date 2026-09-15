# AGENTS.md - [PROJECT_NAME]

## Project Overview

[PROJECT_NAME] is a Python project scaffolded from RapidWebs Enterprise standards.

## Quick Commands

```bash
# Install
make install

# Test
make test
make coverage

# Lint
make lint
make typing
make check

# Format
make format
```

## Documentation

- [Spec Templates](docs/templates/spec-template.md)
- [ADR Templates](docs/templates/adr-template.md)
- [Implementation Plans](docs/templates/implementation-plan-template.md)
- [Research Notes](docs/research/)

## Architecture Decision Records

ADRs are stored in `docs/adrs/`. See [ADR patterns](docs/research/adr-patterns.md) for guidance.

## OpenSpec

Features follow spec-driven development. See [OpenSpec patterns](docs/research/openspec-specs.md).

## Conventions

- Python 3.10+
- src-layout package structure
- Ruff for linting/formatting
- mypy for type checking
- pytest for testing
- Keep a Changelog for version history
- Conventional Commits for commit messages

## Code Style

- Line length: 100 characters
- Type hints required for public APIs
- Google-style docstrings
- No mutable default arguments
- Specific exception handling

## Testing

- Minimum 80% coverage
- Unit tests for logic
- Integration tests for external deps
- E2E tests for CLI if applicable

## Git Workflow

- Branch naming: `feat/description`, `fix/description`, `docs/description`
- PRs require review
- Squash merge to main
- Semantic versioning
