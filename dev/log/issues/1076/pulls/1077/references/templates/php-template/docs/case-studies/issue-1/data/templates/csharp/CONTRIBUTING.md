# Contributing to csharp-ai-driven-development-pipeline-template

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR-USERNAME/csharp-ai-driven-development-pipeline-template.git
   cd csharp-ai-driven-development-pipeline-template
   ```

2. **Install .NET SDK**

   Install .NET 8.0 SDK from [dotnet.microsoft.com](https://dotnet.microsoft.com/download).

3. **Install Bun** (for helper scripts)

   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```

4. **Restore dependencies and build**

   ```bash
   dotnet restore
   dotnet build
   ```

5. **Install pre-commit hooks** (optional but recommended)

   ```bash
   pip install pre-commit
   pre-commit install
   ```

## Development Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**

   - Write code following the project's style guidelines
   - Add tests for any new functionality
   - Update documentation as needed

3. **Run quality checks**

   ```bash
   # Format code
   dotnet format

   # Build with warnings as errors
   dotnet build --configuration Release /warnaserror

   # Check file sizes
   bun run scripts/check-file-size.mjs

   # Run all checks together
   dotnet format --verify-no-changes && dotnet build --configuration Release /warnaserror && bun run scripts/check-file-size.mjs
   ```

4. **Run tests**

   ```bash
   # Run all tests
   dotnet test

   # Run tests with verbose output
   dotnet test --verbosity normal

   # Run tests with coverage
   dotnet test --collect:"XPlat Code Coverage"

   # Run a specific test
   dotnet test --filter "FullyQualifiedName~TestName"
   ```

5. **Add a changeset**

   For any user-facing changes, create a changeset file in `.changeset/`:

   ```bash
   # Create a changeset file
   cat > .changeset/my-change.md << 'EOF'
   ---
   'MyPackage': patch
   ---

   Description of your changes
   EOF
   ```

   **Changeset format:**
   - First line after `---`: `'PackageName': version_type`
   - Version types: `major`, `minor`, or `patch`
   - Description after the second `---`

   **Why changesets?** This workflow:
   - Prevents merge conflicts in CHANGELOG.md
   - Ensures version bumps happen safely after merge
   - Matches the JavaScript Changesets workflow

6. **Commit your changes**

   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

   Pre-commit hooks will automatically run and check your code.

7. **Push and create a Pull Request**

   ```bash
   git push origin feature/my-feature
   ```

   Then create a Pull Request on GitHub.

## Code Style Guidelines

This project uses:

- **EditorConfig** for code style configuration
- **.NET Analyzers** for static analysis
- **dotnet format** for code formatting
- **xUnit** for testing

### Code Standards

- Follow C# conventions and best practices
- Use XML documentation comments (`///`) for all public APIs
- Write tests for all new functionality
- Keep functions focused and reasonably sized
- Keep files under 1000 lines
- Use meaningful variable and function names
- Prefer file-scoped namespaces
- Use expression-bodied members where appropriate

### Documentation Format

Use XML documentation comments:

```csharp
/// <summary>
/// Brief description of the method.
/// </summary>
/// <param name="arg1">Description of arg1.</param>
/// <param name="arg2">Description of arg2.</param>
/// <returns>Description of return value.</returns>
/// <exception cref="ArgumentException">When argument is invalid.</exception>
/// <example>
/// <code>
/// var result = MyMethod(1, 2);
/// Console.WriteLine(result); // Output: 3
/// </code>
/// </example>
public int MyMethod(int arg1, int arg2)
{
    return arg1 + arg2;
}
```

## Testing Guidelines

- Write tests for all new features
- Maintain or improve test coverage
- Use descriptive test names
- Organize tests using nested classes when appropriate
- Use `[Theory]` for data-driven tests

Example test structure:

```csharp
public class MyFeatureTests
{
    public class WhenDoingSomething
    {
        [Fact]
        public void Should_ReturnExpectedResult()
        {
            // Arrange
            var input = "test";

            // Act
            var result = MyFeature.DoSomething(input);

            // Assert
            Assert.Equal("expected", result);
        }

        [Theory]
        [InlineData("input1", "output1")]
        [InlineData("input2", "output2")]
        public void Should_HandleVariousInputs(string input, string expected)
        {
            var result = MyFeature.DoSomething(input);

            Assert.Equal(expected, result);
        }
    }
}
```

## Pull Request Process

1. Ensure all tests pass locally
2. Update documentation if needed
3. Add a changeset file (see step 5 in Development Workflow)
4. Ensure the PR description clearly describes the changes
5. Link any related issues in the PR description
6. Wait for CI checks to pass (including changeset validation)
7. Address any review feedback

## Changesets Workflow

This project uses a changesets workflow similar to [Changesets](https://github.com/changesets/changesets) in JavaScript.

### Creating a Changeset

```bash
# Create a changeset file
cat > .changeset/my-change.md << 'EOF'
---
'MyPackage': patch
---

Description of your changes
EOF
```

### Version Types

- `major` - Breaking changes (1.x.x -> 2.0.0)
- `minor` - New features, backwards compatible (x.1.x -> x.2.0)
- `patch` - Bug fixes, backwards compatible (x.x.1 -> x.x.2)

### How It Works

1. Each PR adds a changeset file in `.changeset/`
2. CI validates that exactly one changeset is added per PR
3. When merged to main, the release workflow:
   - Merges multiple changesets (if any) using highest bump type
   - Updates version in csproj and CHANGELOG.md
   - Creates git tag and pushes changes
   - Publishes to NuGet and creates GitHub release

### Benefits

- **No merge conflicts**: Multiple PRs can add changesets independently
- **Version safety**: Version bumps happen after merge, not before
- **Consistent with JS/Python templates**: Same workflow across all templates

## Project Structure

```
.
├── .changeset/           # Changesets configuration
│   ├── config.json       # Changeset settings
│   ├── README.md         # Changeset instructions
│   └── *.md              # Individual changesets
├── .github/workflows/    # GitHub Actions CI/CD
├── examples/             # Usage examples
├── scripts/              # Utility scripts (.mjs)
├── src/MyPackage/        # Source code
│   ├── Calculator.cs     # Example implementation
│   ├── PackageInfo.cs    # Package version info
│   └── MyPackage.csproj  # Library project
├── tests/MyPackage.Tests/ # Test files
├── .editorconfig         # Code style configuration
├── .pre-commit-config.yaml # Pre-commit hooks
├── Directory.Build.props # Shared build properties
├── MyPackage.sln         # Solution file
├── CHANGELOG.md          # Project changelog
├── CONTRIBUTING.md       # This file
└── README.md             # Project README
```

## Release Process

This project uses semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Releases are triggered automatically when changesets are merged to main, or manually via workflow_dispatch:

**Automatic Release:**
1. PR with changeset is merged to main
2. Release workflow detects and processes changesets
3. Version is bumped, changelog updated, tag created
4. Package published to NuGet, GitHub release created

**Manual Release:**
1. Go to Actions > CI/CD Pipeline > Run workflow
2. Select release mode (`instant` or `changeset-pr`)
3. Choose bump type and optional description

## Getting Help

- Open an issue for bugs or feature requests
- Use discussions for questions and general help
- Check existing issues and PRs before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

Thank you for contributing!
