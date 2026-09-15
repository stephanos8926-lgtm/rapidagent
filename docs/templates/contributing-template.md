# Contributing to [PROJECT_NAME]

Thank you for your interest in contributing to [PROJECT_NAME]! We welcome contributions of all kinds — bug fixes, new features, documentation improvements, and performance optimizations.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Adding a New Tool](#adding-a-new-tool)

## Code of Conduct

This project adheres to the [Contributor Covenant](CODE_OF_CONDUCT.md).
By participating, you agree to uphold this code. Report unacceptable behavior to [AUTHOR_EMAIL].

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork: `git clone https://github.com/[ORG]/[PROJECT_NAME].git`
3. **Set up** a development environment (see below).
4. **Create a branch** for your changes: `git checkout -b feat/my-feature`
5. **Make changes** following the coding standards.
6. **Run tests** before committing.
7. **Push** and open a pull request.

## Development Setup

### Prerequisites

- Python 3.10+
- [uv](https://docs.astral.sh/uv/) (recommended) or pip

### Install

```bash
# Using uv (recommended)
uv sync --all-extras

# Using pip
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

### Pre-commit Hooks

```bash
pre-commit install
pre-commit run --all-files  # Verify everything passes
```

## Coding Standards

### Python

- **Line length**: 100 characters maximum
- **Formatting**: Ruff formatter (matching `ruff format`)
- **Linting**: All code must pass `ruff check` with no errors
- **Type hints**: Required for all public functions and methods
- **Docstrings**: Google-style docstrings for all public APIs

### Tool Implementation Pattern

Every tool follows a consistent pattern:

```python
def _tool_my_new_tool(params: dict[str, Any]) -> dict[str, Any]:
    """Brief description of what the tool does.

    Args:
        param_name: Description (required)
        optional_param: Description (default: value)

    Returns:
        Result dictionary with key-value pairs.
    """
    # Implementation here
    return {"status": "success", "data": result}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new feature
fix: resolve bug
docs: update documentation
refactor: restructure code
test: add tests
chore: maintenance tasks
```

## Testing

Run the test suite before submitting:

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=[package_name] --cov-report=term-missing

# Run specific test file
pytest tests/test_feature.py
```

Tests should have:
- At least 80% code coverage
- Clear test names describing the behavior
- Both happy path and edge cases

## Pull Request Process

1. **Update documentation** if you change functionality
2. **Add tests** for new features
3. **Ensure CI passes** — all checks must be green
4. **Write a good PR description** — explain what and why
5. **Request review** from maintainers

### PR Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] ADR updated (if significant decision)
- [ ] No debug print statements
- [ ] Type hints present on new public APIs

## Architecture Decision Records

Significant architectural decisions are documented as ADRs in `docs/adrs/`. See [ADR patterns](docs/research/adr-patterns.md) for guidance.

## Spec-Driven Development

Complex features use OpenSpec. See [OpenSpec patterns](docs/research/openspec-specs.md) for guidance.

## Reporting Bugs

Please read our [issue creation guide](../.github/ISSUE_TEMPLATE/bug_report.md) before submitting a bug report.

## Questions?

- Join our [Discord](https://discord.gg/rapidwebs) for discussion
- Open an issue for bugs or feature requests
