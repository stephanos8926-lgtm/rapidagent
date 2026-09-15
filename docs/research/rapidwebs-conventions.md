# RapidWebs Python Project Conventions

## Overview
This document captures the conventions and patterns used in RapidWebs Enterprise projects (ast-tools, hermes-agent, NexusAgent, etc.).

## Technology Stack

### Build Tooling
- **uv** - Fast Python package manager and resolver (recommended over pip)
- **hatchling** - Build backend for pyproject.toml
- **pytest** - Testing framework

### Code Quality
- **ruff** - Linter and formatter (replaces flake8, isort, black)
- **mypy** - Static type checker (strict mode)
- **pre-commit** - Git hook management

### Documentation
- **Markdown** - All documentation in Markdown
- **OpenSpec** - Spec-driven development (for complex features)
- **ADR** - Architecture Decision Records (Nygard format)

## Directory Structure

### Standard Layout
```
project/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── docs/
│   ├── adrs/
│   ├── research/
│   └── specs/
├── src/
│   └── package_name/
│       ├── __init__.py
│       ├── cli.py
│       └── core/
│           ├── __init__.py
│           └── engine.py
├── tests/
│   ├── test_core.py
│   └── conftest.py
├── pyproject.toml
├── Makefile
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE
└── AGENTS.md
```

### Key Directories
- `src/` - Source code (src-layout)
- `tests/` - Test suite (separate from source)
- `docs/adrs/` - Architecture Decision Records
- `docs/research/` - Research notes and findings
- `docs/specs/` - OpenSpec specifications (if used)

## Coding Standards

### Python Version
- **Minimum**: Python 3.10
- **Target**: 3.10, 3.11, 3.12, 3.13

### Line Length
- **Maximum**: 100 characters
- **Enforced by**: ruff

### Type Hints
- **Required** for all public functions and methods
- **Optional** for private/internal functions
- **Strict mode** enabled in mypy

### Docstrings
- **Google style** for public APIs
- **Required** for all exported functions/classes
- **Examples** encouraged in docstrings

### Error Handling
- **Specific exceptions** preferred over bare `except`
- **Context** preserved in custom exceptions
- **Logging** instead of printing

## Commit Message Convention

### Format
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style (formatting, semicolons, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

### Examples
```
feat(cli): add --verbose flag for debug output

fix(parser): handle empty input gracefully

docs: update ADR for caching strategy
```

## Testing Standards

### Test Organization
```
tests/
├── test_core.py           # Unit tests for core modules
├── test_cli.py            # CLI integration tests
├── conftest.py            # Shared fixtures
└── fixtures/              # Test data
    └── sample.json
```

### Coverage Requirements
- **Minimum**: 80% line coverage
- **Fail threshold**: Configured in pyproject.toml
- **Report format**: `term-missing` for CI, `html` for local

### Test Naming
- `test_` prefix for test functions
- Descriptive names: `test_should_return_error_on_invalid_input`
- Group related tests: `class TestCore:` with `def test_method(self):`

## Configuration Files

### pyproject.toml
All project configuration in one file:
- Build system
- Dependencies
- Tool settings (ruff, mypy, pytest, coverage)
- Entry points

### Makefile
Standard targets:
- `make install` - Install dependencies
- `make format` - Format code
- `make lint` - Run linter
- `make test` - Run tests
- `make coverage` - Run tests with coverage
- `make check` - Run lint + test
- `make clean` - Clean build artifacts

### .pre-commit-config.yaml
Hooks run before each commit:
- ruff (lint + fix)
- ruff-format
- trailing-whitespace
- end-of-file-fixer
- check-yaml
- check-merge-conflict

## Documentation Standards

### README.md
Required sections:
1. Project title and description
2. Badges (CI, coverage, version, license)
3. Installation
4. Usage examples
5. Development setup
6. Contributing guidelines
7. License

### CHANGELOG.md
Format: Keep a Changelog + Semantic Versioning
```markdown
## [Unreleased]

### Added
- New feature

### Changed
- Modified behavior

### Fixed
- Bug fix

### Removed
- Deprecated feature
```

### AGENTS.md
AI agent guidance (project-specific instructions, patterns, and warnings)

## GitHub Integration

### Branch Naming
- `feat/<description>` - New features
- `fix/<description>` - Bug fixes
- `docs/<description>` - Documentation
- `chore/<description>` - Maintenance

### Pull Request Template
```markdown
## Description
[What changed and why]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] ADR updated (if significant decision)
```

### Issue Templates
- `BUG_REPORT.md` - Bug report template
- `FEATURE_REQUEST.md` - Feature request template

## Security Practices

### Secrets Management
- Never commit secrets to repository
- Use environment variables or `.env` files (gitignored)
- Use GitHub Secrets for CI/CD
- Audit dependencies with `pip-audit`

### Dependency Security
- Pin dependency versions
- Regular security audits
- Use Dependabot for automated updates
- Review changelogs before updating major versions

## Performance Considerations

### Memory Management
- Monitor memory usage in long-running processes
- Use generators for large datasets
- Implement resource cleanup (context managers)

### CPU Optimization
- Profile before optimizing
- Prefer vectorized operations (numpy, pandas)
- Use appropriate data structures

## References
- [ast-tools Project](https://github.com/stephanos8926-lgtm/ast-tools)
- [hermes-agent Project](https://github.com/stephanos8926-lgtm/hermes-agent)
- [RapidWebs Enterprise Standards](https://rapidwebs.org)
