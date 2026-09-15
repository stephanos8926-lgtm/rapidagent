"""Tests for the project scaffold script."""


def test_slug_to_package_name():
    """Test package name generation from project name."""
    from scripts.scaffold_project import slug_to_package_name

    assert slug_to_package_name("my-project") == "my_project"
    assert slug_to_package_name("My Project") == "my_project"
    assert slug_to_package_name("my_project") == "my_project"
    assert slug_to_package_name("123project") == "project"


def test_project_validation():
    """Test project name validation."""

    from scripts.scaffold_project import slug_to_project_name

    assert slug_to_project_name("My Project") == "my-project"
    assert slug_to_project_name("my_project") == "my-project"


def test_template_replacements():
    """Test template placeholder replacement."""
    from scripts.scaffold_project import read_template

    content = "Hello [PROJECT_NAME], by [AUTHOR_NAME]"
    replacements = {"[PROJECT_NAME]": "test", "[AUTHOR_NAME]": "John"}
    result = read_template(content, replacements)
    assert "test" in result
    assert "John" in result
