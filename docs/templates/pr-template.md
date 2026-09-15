# Pull Request Template

## Description

Please include a summary of the change and which issue is fixed. Please also include relevant motivation and context.

Fixes # (issue)

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test improvement

## Checklist

- [ ] I have read the [CONTRIBUTING.md](../../CONTRIBUTING.md) document
- [ ] My code follows the code style of this project
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have updated/extended relevant documentation
- [ ] I have added an entry to [CHANGELOG.md](../../CHANGELOG.md)
- [ ] I have written an ADR if this is a significant architectural decision
- [ ] My commits follow [Conventional Commits](https://www.conventionalcommits.org/)

## Testing

Describe the tests you ran to verify your changes.

```bash
# Example test commands
pytest tests/test_feature.py -v
pytest --cov=src --cov-report=term-missing
ruff check .
mypy src/
```

## Screenshots (if appropriate)

Add screenshots of UI changes if applicable.

## Additional Notes

Any additional information about the PR that might be helpful for reviewers.
