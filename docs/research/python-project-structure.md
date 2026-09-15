# Modern Python Project Structure

## Layout Options

### src-layout (Recommended)
```
project_root/
├── pyproject.toml
├── src/
│   └── package_name/
│       ├── __init__.py
│       ├── module1.py
│       └── submodule/
│           ├── __init__.py
│           └── module2.py
├── tests/
│   ├── test_module1.py
│   └── test_submodule/
│       └── test_module2.py
└── docs/
```

**Advantages:**
- Prevents importing from source tree directly (catches packaging issues early)
- Automatic package discovery with setuptools
- Required for PEP 420 namespace packages

**Disadvantages:**
- Cannot run Python REPL from project root without editable install
- Need `python -m package_name` or editable install for development

### flat-layout
```
project_root/
├── pyproject.toml
├── package_name/
│   ├── __init__.py
│   └── module.py
├── tests/
└── docs/
```

**Advantages:**
- Simple REPL access
- Works without editable install

**Disadvantages:**
- Can accidentally import from source tree during testing
- Harder to catch packaging issues

## pyproject.toml (PEP 621)

### Minimal Configuration
```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "package-name"
version = "0.1.0"
description = "One-line description"
readme = "README.md"
license = {text = "MIT"}
requires-python = ">=3.10"
authors = [
    {name = "Author Name", email = "author@example.com"},
]
keywords = ["keyword1", "keyword2"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
]

dependencies = [
    "dependency1>=1.0",
    "dependency2>=2.0,<3.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0",
    "ruff>=0.4.0",
    "mypy>=1.0",
]
test = [
    "pytest>=8.0",
    "pytest-cov>=4.1",
]

[project.scripts]
cli = "package_name.cli:main"
```

### Tool Configuration
```toml
[tool.ruff]
line-length = 100
target-version = "py310"

[tool.ruff.lint]
select = ["E", "F", "W", "I", "N", "UP", "B", "C4"]

[tool.mypy]
python_version = "3.10"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
addopts = "-v --cov=src --cov-report=term-missing"

[tool.coverage.run]
source = ["src"]

[tool.coverage.report]
fail_under = 80
```

## Package Discovery (setuptools)

### For src-layout
```toml
[tool.setuptools.packages.find]
where = ["src"]
include = ["package_name*"]
namespaces = false
```

### For flat-layout
```toml
[tool.setuptools.packages.find]
include = ["package_name*"]
exclude = ["package_name.tests*"]
```

## Dependency Management

### Using uv (Recommended)
```bash
# Create virtual environment
uv venv

# Install dependencies
uv pip install -e ".[dev,test]"

# Run project
uv run python -m package_name
```

### Using pip
```bash
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev,test]"
```

## Version Management

### Single-source version (recommended)
```python
# src/package_name/__init__.py
__version__ = "0.1.0"
```

```toml
[tool.setuptools_scm]
# Auto-detect version from git tags
```

### Dynamic version
```toml
[project]
dynamic = ["version"]
```

## Build Backends Comparison

| Backend | Features | Complexity |
|---------|----------|------------|
| **hatchling** | Fast, modern, extensible | Low |
| **setuptools** | Universal support | Medium |
| **flit** | Simple, PEP-compliant | Low |
| **poetry** | Dependency resolution, lock files | High |

**Recommendation**: Use `hatchling` for new projects.

## References
- [PEP 621 – Storing project metadata in pyproject.toml](https://peps.python.org/pep-0621/)
- [Python Packaging User Guide](https://packaging.python.org/)
- [Hatchling Documentation](https://hatch.pypa.io/)
- [uv Documentation](https://docs.astral.sh/uv/)
