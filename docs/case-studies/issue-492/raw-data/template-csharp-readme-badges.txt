# csharp-ai-driven-development-pipeline-template

A comprehensive template for AI-driven C# development with full CI/CD pipeline support.

[![CI/CD Pipeline](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/actions)
[![.NET Version](https://img.shields.io/badge/.NET-8.0-blue.svg)](https://dotnet.microsoft.com/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Features

- **.NET 8.0 support**: Works with the latest .NET LTS version
- **Cross-platform testing**: CI runs on Ubuntu, macOS, and Windows
- **Comprehensive testing**: xUnit tests with coverage reporting
- **Code quality**: EditorConfig + .NET analyzers with warnings as errors
- **Pre-commit hooks**: Automated code quality checks before commits
- **CI/CD pipeline**: GitHub Actions with multi-platform support
- **Changesets workflow**: Version-safe changelog management (like JavaScript Changesets)
- **Release automation**: Automatic NuGet publishing and GitHub releases
- **API documentation**: DocFX build and GitHub Pages deployment on every push to `main`

## Quick Start

### Using This Template

1. Click "Use this template" on GitHub to create a new repository
2. Clone your new repository
3. Update `src/MyPackage/MyPackage.csproj` with your package name and description
4. Rename the solution and project files as needed
5. Update imports in tests and examples
6. Build and start developing!

### Development Setup

```bash
# Clone the repository
git clone https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template.git
cd csharp-ai-driven-development-pipeline-template

# Build the project
dotnet build

# Run tests
dotnet test

# Run the example
dotnet run --project examples/BasicUsage
```

### Running Tests

```bash
# Run all tests
dotnet test

# Run tests with verbose output
dotnet test --verbosity normal

# Run tests with coverage
dotnet test --collect:"XPlat Code Coverage"

# Run a specific test
dotnet test --filter "FullyQualifiedName~CalculatorTests"
```

### Code Quality Checks

```bash
# Format code
dotnet format

# Check formatting (CI style)
dotnet format --verify-no-changes

# Build with warnings as errors
dotnet build --configuration Release /warnaserror

# Check file size limits
bun run scripts/check-file-size.mjs

# Run all checks
