# CI/CD Patterns for Python Projects

## GitHub Actions Workflow Structure

### Basic CI Workflow
```yaml
name: CI

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

permissions:
  contents: read

concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  quality:
    name: Python ${{ matrix.python-version }}
    runs-on: ubuntu-latest
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        python-version: ['3.10', '3.11', '3.12', '3.13']
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Python ${{ matrix.python-version }}
        uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
          cache: pip
          cache-dependency-path: pyproject.toml
      
      - name: Install uv
        uses: astral-sh/setup-uv@v3
      
      - name: Install dependencies
        run: uv pip install -e ".[dev,test,lint]"
      
      - name: Lint with ruff
        run: ruff check .
      
      - name: Format check
        run: ruff format --check .
      
      - name: Type check with mypy
        run: mypy src/
      
      - name: Run tests
        run: pytest --cov=src --cov-report=xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          file: ./coverage.xml
```

## Pre-commit Hooks

### Configuration
```yaml
repos:
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.4.0
    hooks:
      - id: ruff
        args: [--fix]
      - id: ruff-format

  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
      - id: check-merge-conflict

  - repo: https://github.com/commitizen-tools/commitizen
    rev: v3.13.0
    hooks:
      - id: commitizen
```

### Installation
```bash
pre-commit install
pre-commit run --all-files  # Verify everything passes
```

## Release Automation

### Using python-semantic-release
```yaml
name: Release

on:
  push:
    branches: [main, master]

permissions:
  contents: read
  packages: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      
      - name: Install semantic-release
        run: pip install python-semantic-release
      
      - name: Generate changelog and version
        run: semantic-release version
      
      - name: Publish to PyPI
        run: semantic-release publish
        env:
          PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
```

## Dependency Management

### Dependabot Configuration
```yaml
version: 2
updates:
  - package-ecosystem: "pip"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

## Security Scanning

### CodeQL
```yaml
- name: Initialize CodeQL
  uses: github/codeql-action/init@v3
  with:
    languages: python
    
- name: Perform CodeQL Analysis
  uses: github/codeql-action/analyze@v3
```

### pip-audit
```yaml
- name: Security audit
  run: pip-audit --require-hashes
```

## Matrix Testing

### Strategy Pattern
```yaml
strategy:
  fail-fast: false
  matrix:
    python-version: ['3.10', '3.11', '3.12', '3.13']
    os: [ubuntu-latest, windows-latest, macos-latest]
    exclude:
      # Skip macOS for Python 3.10 if not needed
      - python-version: '3.10'
        os: macos-latest
```

## Best Practices

### 1. Cache Dependencies
```yaml
- name: Cache pip packages
  uses: actions/cache@v4
  with:
    path: ~/.cache/pip
    key: ${{ runner.os }}-pip-${{ hashFiles('pyproject.toml') }}
    restore-keys: |
      ${{ runner.os }}-pip-
```

### 2. Timeout Enforcement
```yaml
timeout-minutes: 15
```

### 3. Concurrency Control
```yaml
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

### 4. Minimal Permissions
```yaml
permissions:
  contents: read
```

## References
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Pre-commit Documentation](https://pre-commit.com/)
- [python-semantic-release](https://python-semantic-release.readthedocs.io/)
- [Codecov GitHub Action](https://github.com/codecov/codecov-action)
