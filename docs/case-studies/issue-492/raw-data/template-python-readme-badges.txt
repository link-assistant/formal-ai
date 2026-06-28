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
