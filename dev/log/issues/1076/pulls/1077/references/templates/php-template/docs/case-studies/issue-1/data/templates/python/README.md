# python-ai-driven-development-pipeline-template

A comprehensive template for AI-driven Python development with full CI/CD pipeline support.

[![CI/CD Pipeline](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/workflows/CI/CD%20Pipeline/badge.svg)](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/actions)
[![Python Version](https://img.shields.io/badge/python-3.9%2B-blue.svg)](https://www.python.org/downloads/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Features

- **Multi-version Python support**: Works with Python 3.9-3.13
- **Comprehensive testing**: pytest with async support and coverage reporting
- **Code quality**: Ruff (linting + formatting) + mypy (type checking)
- **Pre-commit hooks**: Automated code quality checks before commits
- **CI/CD pipeline**: GitHub Actions CI/CD with Python 3.13
- **Changelog management**: Scriv for conflict-free changelog (like Changesets in JS)
- **Release automation**: Automatic PyPI publishing and GitHub releases
- **API documentation**: Sphinx + GitHub Pages deploy on push to `main`

## Quick Start

### Using This Template

1. Click "Use this template" on GitHub to create a new repository
2. Clone your new repository
3. Update `pyproject.toml` with your package name and description
4. Rename `src/my_package` to your package name
5. Update imports in tests and examples
6. Install dependencies and start developing!

### Development Setup

```bash
# Clone the repository
git clone https://github.com/link-foundation/python-ai-driven-development-pipeline-template.git
cd python-ai-driven-development-pipeline-template

# Create a virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install in editable mode with development dependencies
pip install -e ".[dev]"

# Install pre-commit hooks
pip install pre-commit
pre-commit install
```

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=src --cov-report=term --cov-report=html

# Run specific test file
pytest tests/test_my_package.py

# Run with verbose output
pytest -v
```

### Code Quality Checks

```bash
# Lint code (check for issues)
ruff check .

# Format code
ruff format .

# Type check
mypy src/

# Check file size limits
python scripts/check_file_size.py

# Run all checks
ruff check . && ruff format --check . && mypy src/ && python scripts/check_file_size.py
```

## Project Structure

```
.
├── .github/
│   └── workflows/
│       ├── docs.yml            # Sphinx build + GitHub Pages deploy
│       └── release.yml         # CI checks + release automation (PyPI + GitHub)
├── changelog.d/                # Changelog fragments (like .changeset/)
│   ├── README.md               # Fragment instructions
│   └── *.md                    # Individual changelog entries
├── docs/                       # Sphinx documentation source
│   ├── conf.py                 # Sphinx configuration
│   ├── index.md                # Documentation landing page
│   ├── api.md                  # Auto-generated API reference
│   └── requirements.txt        # Documentation build dependencies
├── examples/
│   └── basic_usage.py          # Usage examples
├── scripts/
│   ├── check_file_size.py      # File size validation script
│   ├── bump_version.py         # Version bumping utility
│   ├── version_and_commit.py   # CI/CD version management
│   ├── publish_to_pypi.py      # PyPI publishing script
│   └── create_github_release.py # GitHub release creation
├── src/
│   └── my_package/
│       ├── __init__.py         # Package entry point
│       └── py.typed            # Type marker file
├── tests/
│   ├── __init__.py
│   └── test_my_package.py      # Test suite
├── .gitignore                  # Git ignore patterns
├── .pre-commit-config.yaml     # Pre-commit hooks configuration
├── .ruff.toml                  # Ruff additional configuration
├── pyproject.toml              # Project configuration and dependencies
├── CHANGELOG.md                # Project changelog
├── CONTRIBUTING.md             # Contribution guidelines
├── LICENSE                     # Unlicense (public domain)
└── README.md                   # This file
```

## Design Choices

### Package Management

This template uses modern Python packaging standards:

- **pyproject.toml**: Single source of truth for project configuration
- **hatchling**: Modern build backend (PEP 517)
- **src layout**: Prevents accidental imports from source directory
- **py.typed**: Marks package as type-hinted for mypy

### Code Quality Tools

- **Ruff**: Ultra-fast Python linter and formatter (replaces flake8, black, isort)
  - Configured for strict code quality standards
  - Integrates with pre-commit hooks
  - Consistent formatting across the project

- **mypy**: Static type checker
  - Strict mode enabled for maximum type safety
  - Ensures code correctness before runtime

- **pytest**: Modern testing framework
  - Support for async tests via pytest-asyncio
  - Coverage reporting via pytest-cov
  - Organized test structure with classes

### Pre-commit Hooks

Automated checks run before each commit:

1. Basic checks (trailing whitespace, file endings, etc.)
2. Ruff linting and formatting
3. mypy type checking

This ensures code quality is maintained throughout development.

### Changelog Management (Scriv)

This template uses [Scriv](https://scriv.readthedocs.io/) for changelog management, which works similarly to [Changesets](https://github.com/changesets/changesets) in JavaScript projects:

- **Fragment-based**: Each PR adds a changelog fragment to `changelog.d/`
- **Conflict-free**: Multiple PRs can add fragments without merge conflicts
- **Auto-collection**: Fragments are automatically merged during release
- **Category-based**: Supports Added, Changed, Deprecated, Removed, Fixed, Security

```bash
# Create a changelog fragment (similar to `npx changeset`)
scriv create

# View pending fragments
ls changelog.d/*.md
```

### CI/CD Pipeline

The GitHub Actions workflow provides:

1. **Linting**: Ruff linting, formatting, and mypy type checking
2. **Changelog check**: Warns if PRs are missing changelog fragments
3. **Testing**: Python 3.13 test suite
4. **Building**: Package building and validation
5. **Coverage**: Automatic upload to Codecov

### API Documentation

API documentation is built with [Sphinx](https://www.sphinx-doc.org/) and deployed
to GitHub Pages on every push to `main`. Pull requests build the docs (without
deploying) to catch regressions before they merge.

```bash
# Install docs dependencies
pip install -e ".[docs]"

# Build locally (output goes to _site/)
sphinx-build -W --keep-going -b html docs _site

# Preview in a browser
python -m http.server --directory _site 8000
```

The Sphinx config in `docs/conf.py` autodiscovers `src/my_package` via
`sphinx.ext.autodoc` + `sphinx.ext.autosummary` and renders Google-style
docstrings with `sphinx.ext.napoleon`. When you bootstrap a new repository from
this template, update `project`, `author`, and the autosummary target in
`docs/conf.py` and `docs/api.md` to point at your package.

#### Deploying API documentation

The `Docs` workflow (`.github/workflows/docs.yml`) builds on every push and
pull request, and deploys to GitHub Pages only on `push` to `main` (matching
the JS and Rust template patterns; see
[link-foundation/relative-meta-logic#170](https://github.com/link-foundation/relative-meta-logic/pull/170)
for the bug this guards against).

**One-time setup per repository**: open `Settings → Pages` and set
`Source = GitHub Actions`. Without this, the first deploy run fails on
`actions/deploy-pages` with `Get Pages site failed`. This cannot be configured
from a workflow.

### Preview regeneration parity

[Issue #9](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/9)
tracks parity with the browser preview-regeneration pattern from the JavaScript
template. This Python template does not ship a browser-rendered example app, so
it intentionally does not install Playwright or add a screenshot workflow yet.
When such a surface is added, use the Python porting checklist in
[`docs/preview-regeneration.md`](docs/preview-regeneration.md).

### Release Automation

The release workflow (`release.yml`) provides:

1. **Integrated CI checks**: Runs lint, test, and build before any release
2. **Auto-release on push**: Detects version changes and publishes automatically
3. **Manual release**: Trigger releases via workflow_dispatch
4. **Fragment collection**: Automatically collects changelog fragments
5. **PyPI publishing**: OIDC trusted publishing (no tokens needed)
6. **GitHub releases**: Automatic creation with CHANGELOG content

**Important**: All releases require passing CI checks (lint + test + build). No release will ever happen without passing tests, ensuring code quality and stability.

## Configuration

### Updating Package Name

After creating a repository from this template:

1. Update `pyproject.toml`:
   - Change `name` field
   - Update `project.urls`
   - Update `tool.hatch.build.targets.wheel.packages`

2. Rename `src/my_package/` directory to your package name

3. Update imports:
   - `tests/test_my_package.py`
   - `examples/basic_usage.py`
   - `.ruff.toml` (known-first-party)

### Ruff Configuration

Customize Ruff in `pyproject.toml` under `[tool.ruff]`. Current configuration:

- 88-character line length (Black-compatible)
- Comprehensive linting rules (E, W, F, I, N, UP, B, etc.)
- Strict equality enforcement
- Automatic import sorting

### mypy Configuration

Configured in `pyproject.toml` under `[tool.mypy]`:

- Strict mode enabled
- No implicit optionals
- Warn on unused ignores
- Full type checking coverage

### pytest Configuration

Configured in `pyproject.toml` under `[tool.pytest.ini_options]`:

- Test discovery in `tests/` directory
- Source path includes `src/`
- Strict marker enforcement
- Coverage configuration included

## Scripts Reference

| Script                         | Description                              |
| ------------------------------ | ---------------------------------------- |
| `pytest`                       | Run all tests                            |
| `pytest --cov=src`             | Run tests with coverage                  |
| `ruff check .`                 | Lint code                                |
| `ruff format .`                | Format code                              |
| `mypy src/`                    | Type check code                          |
| `python scripts/check_file_size.py` | Check file size limits             |
| `pre-commit run --all-files`   | Run all pre-commit hooks                 |
| `scriv create`                 | Create a changelog fragment              |
| `scriv collect --version X.Y.Z`| Collect fragments into CHANGELOG.md      |

## Example Usage

```python
from my_package import add, multiply, delay
import asyncio

# Basic arithmetic
result = add(2, 3)  # 5
product = multiply(2, 3)  # 6

# Async operations
async def main():
    await delay(1.0)  # Wait for 1 second

asyncio.run(main())
```

See `examples/basic_usage.py` for more examples.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes and add tests
4. Run quality checks: `ruff check . && ruff format . && mypy src/ && pytest`
5. Commit your changes (pre-commit hooks will run automatically)
6. Push and create a Pull Request

## Testing

This project maintains high test coverage and uses pytest for testing:

- Unit tests for all functions
- Async test support
- Coverage reporting
- Cross-platform compatibility testing

## License

[Unlicense](LICENSE) - Public Domain

This is free and unencumbered software released into the public domain. See [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by [js-ai-driven-development-pipeline-template](https://github.com/link-foundation/js-ai-driven-development-pipeline-template).

## Resources

- [Python Packaging Guide](https://packaging.python.org/)
- [pytest Documentation](https://docs.pytest.org/)
- [Ruff Documentation](https://docs.astral.sh/ruff/)
- [mypy Documentation](https://mypy.readthedocs.io/)
- [Pre-commit Documentation](https://pre-commit.com/)
- [Scriv Documentation](https://scriv.readthedocs.io/)
