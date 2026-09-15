#!/usr/bin/env python3
"""
Project scaffold script for RapidWebs Python projects.
Creates a new project from templates with custom project name.
"""

import argparse
import re
import sys
from pathlib import Path

# Placeholder patterns to replace
PLACEHOLDER_PATTERNS = {
    "[PROJECT_NAME]": "{{project_name}}",
    "[project_name]": "{{project_name_snake}}",
    "[package_name]": "{{package_name_snake}}",
    "[AUTHOR_NAME]": "{{author_name}}",
    "[AUTHOR_EMAIL]": "{{author_email}}",
    "[ORG]": "{{org_name}}",
    "[YEAR]": "{{year}}",
    "[LICENSE_TYPE]": "{{license_type}}",
}


def slug_to_package_name(name: str) -> str:
    """Convert project name to valid Python package name."""
    # Replace hyphens and spaces with underscores
    package = name.replace("-", "_").replace(" ", "_")
    # Remove any characters that aren't alphanumeric or underscore
    package = re.sub(r"[^a-zA-Z0-9_]", "", package)
    # Remove leading numbers
    package = re.sub(r"^[0-9]+", "", package)
    return package.lower()


def slug_to_project_name(name: str) -> str:
    """Convert to kebab-case for project name."""
    return name.replace("_", "-").replace(" ", "-").lower()


def read_template(content: str, replacements: dict) -> str:
    """Read template and apply replacements."""
    result = content
    for key, value in replacements.items():
        result = result.replace(key, value)
    return result


def create_project(
    project_name: str,
    output_dir: Path,
    author_name: str,
    author_email: str,
    org_name: str = "",
    license_type: str = "MIT",
):
    """Create a new Python project from templates."""
    # Validate project name
    if not project_name.isidentifier() and project_name.replace("-", "").isalnum():
        pass  # Allow hyphenated names for project
    elif not project_name.replace("-", "").isalnum():
        print(f"Error: '{project_name}' contains invalid characters")
        sys.exit(1)

    package_name = slug_to_package_name(project_name)
    project_slug = slug_to_project_name(project_name)

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Template replacements
    replacements = {
        "{{project_name}}": project_slug,
        "{{project_name_snake}}": package_name,
        "{{package_name_snake}}": package_name,
        "{{author_name}}": author_name,
        "{{author_email}}": author_email,
        "{{org_name}}": org_name or "rapidwebs",
        "{{year}}": "2026",
        "{{license_type}}": license_type,
    }

    # Template sources
    template_base = Path(__file__).parent.parent / "docs" / "templates"

    print(f"Creating project: {project_slug}")
    print(f"Package name: {package_name}")
    print(f"Output directory: {output_dir}")
    print()

    # Create directory structure
    dirs = [
        output_dir / "src" / package_name,
        output_dir / "tests",
        output_dir / "docs" / "adrs",
        output_dir / "docs" / "research",
        output_dir / "docs" / "templates",
        output_dir / ".github" / "workflows",
        output_dir / "scripts",
    ]
    for d in dirs:
        d.mkdir(parents=True, exist_ok=True)
        print(f"  Created: {d}")

    # Create source package init
    init_content = f'''"""{project_slug.title()} - {author_name}"""

__version__ = "0.1.0"
__author__ = "{author_name}"
__email__ = "{author_email}"

from .core import *
'''
    (output_dir / "src" / package_name / "__init__.py").write_text(init_content)
    print(f"  Created: src/{package_name}/__init__.py")

    # Create core module
    core_content = '''"""Core functionality for {project_slug}."""

from typing import Any


def hello(name: str = "World") -> str:
    """Return a greeting message.

    Args:
        name: Name to greet. Defaults to "World".

    Returns:
        Greeting message.
    """
    return f"Hello, {name}!"


def process_data(data: list[Any]) -> list[Any]:
    """Process a list of data items.

    Args:
        data: List of items to process.

    Returns:
        Processed list.
    """
    return sorted(data)
'''
    (output_dir / "src" / package_name / "core.py").write_text(core_content)
    print(f"  Created: src/{package_name}/core.py")

    # Create CLI module
    cli_content = '''"""Command-line interface for {project_slug}."""

import argparse
import sys

from .core import hello


def main() -> int:
    """Main entry point for the CLI."""
    parser = argparse.ArgumentParser(
        prog="{project_slug}",
        description="{project_slug.title()} CLI tool",
    )
    parser.add_argument(
        "--name",
        default="World",
        help="Name to greet (default: World)",
    )
    parser.add_argument(
        "--version",
        action="version",
        version=f"%(prog)s 0.1.0",
    )

    args = parser.parse_args()
    print(hello(args.name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
'''
    (output_dir / "src" / package_name / "cli.py").write_text(cli_content)
    print(f"  Created: src/{package_name}/cli.py")

    # Copy and process template files
    template_files = [
        ("pyproject-toml-template.toml", "pyproject.toml"),
        ("makefile-template", "Makefile"),
        ("gitignore-template", ".gitignore"),
        ("precommit-config-template.yaml", ".pre-commit-config.yaml"),
        ("license-template", "LICENSE"),
        ("code-of-conduct-template.md", "CODE_OF_CONDUCT.md"),
        ("readme-template.md", "README.md"),
        ("changelog-template.md", "CHANGELOG.md"),
        ("spec-template.md", "docs/templates/spec-template.md"),
        ("adr-template.md", "docs/templates/adr-template.md"),
        ("implementation-plan-template.md", "docs/templates/implementation-plan-template.md"),
        ("proposal-template.md", "docs/templates/proposal-template.md"),
    ]

    for src_name, dest_name in template_files:
        src = template_base / src_name
        if src.exists():
            content = src.read_text()
            # Apply project-specific replacements
            content = read_template(content, replacements)
            dest = output_dir / dest_name
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(content)
            print(f"  Created: {dest_name}")

    # Copy CI workflows
    workflow_templates = Path(__file__).parent.parent / ".github" / "workflows"
    if workflow_templates.exists():
        for wf in workflow_templates.glob("*.yml"):
            content = wf.read_text()
            content = read_template(content, replacements)
            (output_dir / ".github" / "workflows" / wf.name).write_text(content)
            print(f"  Created: .github/workflows/{wf.name}")

    # Create CONTRIBUTING.md
    contributing = """# Contributing to {project_slug}

Thank you for your interest in contributing to {project_slug}!

## Code of Conduct

This project adheres to the [Contributor Covenant](CODE_OF_CONDUCT.md).
By participating, you agree to uphold this code.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/{org_name}/{project_slug}.git`
3. Create a branch: `git checkout -b feat/your-feature`
4. Make your changes
5. Run tests: `pytest`
6. Run linter: `ruff check .`
7. Commit using conventional commits
8. Push and open a Pull Request

## Development Setup

```bash
# Install dependencies
make install

# Run tests
make test

# Run linting
make lint

# Run type checking
make typing
```

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new feature
fix: resolve bug
docs: update documentation
refactor: restructure code
test: add tests
chore: maintenance tasks
```

## Pull Requests

- PRs should be small and focused
- Include tests for new features
- Update documentation as needed
- All CI checks must pass

## Reporting Issues

- Use [GitHub Issues](https://github.com/{org_name}/{project_slug}/issues)
- Provide clear reproduction steps
- Include environment details
"""
    (output_dir / "CONTRIBUTING.md").write_text(read_template(contributing, replacements))
    print("  Created: CONTRIBUTING.md")

    # Create AGENTS.md
    agents_md = f"""# AGENTS.md - {project_slug}

## Project Overview

{project_slug.title()} is a Python project scaffolded from RapidWebs Enterprise standards.

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
"""
    (output_dir / "AGENTS.md").write_text(agents_md)
    print("  Created: AGENTS.md")

    # Create empty test file
    test_init = output_dir / "tests" / "__init__.py"
    test_init.touch()
    print("  Created: tests/__init__.py")

    test_main = '''"""Tests for {package_name} core module."""

from {package_name}.core import hello, process_data


def test_hello_default():
    """Test hello with default name."""
    result = hello()
    assert result == "Hello, World!"


def test_hello_custom():
    """Test hello with custom name."""
    result = hello("RapidWebs")
    assert result == "Hello, RapidWebs!"


def test_process_data():
    """Test data processing."""
    data = [3, 1, 4, 1, 5, 9]
    result = process_data(data)
    assert result == [1, 1, 3, 4, 5, 9]
'''
    (output_dir / "tests" / "test_core.py").write_text(read_template(test_main, replacements))
    print("  Created: tests/test_core.py")

    print()
    print(f"Project '{project_slug}' created successfully at {output_dir}")
    print()
    print("Next steps:")
    print(f"  cd {output_dir}")
    print("  make install")
    print("  make check")


def main():
    parser = argparse.ArgumentParser(
        description="Scaffold a new Python project from RapidWebs templates",
    )
    parser.add_argument("project_name", help="Project name (e.g., my-project)")
    parser.add_argument(
        "--output", "-o", default=".", help="Output directory (default: current dir)"
    )
    parser.add_argument("--author-name", required=True, help="Author full name")
    parser.add_argument("--author-email", required=True, help="Author email")
    parser.add_argument("--org", default="", help="Organization/username for GitHub")
    parser.add_argument(
        "--license",
        default="MIT",
        choices=["MIT", "Apache-2.0", "BSD-3-Clause", "GPL-3.0"],
        help="License type (default: MIT)",
    )

    args = parser.parse_args()

    create_project(
        project_name=args.project_name,
        output_dir=Path(args.output),
        author_name=args.author_name,
        author_email=args.author_email,
        org_name=args.org,
        license_type=args.license,
    )


if __name__ == "__main__":
    main()
