# MyPackage Documentation

Welcome to the MyPackage API documentation site, published from this template's
GitHub Pages deployment workflow.

## Quick links

- [API reference](xref:MyPackage) — auto-generated from XML doc comments in `src/`.
- [Roadmap](roadmap/preview-regeneration.md) — deferred work tracked in-repo.
- [Project README](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template#readme)
- [Contributing guide](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/blob/main/CONTRIBUTING.md)

## How this site is built

`docfx.json` at the repository root drives a [DocFX](https://dotnet.github.io/docfx/)
build. The `.github/workflows/docs.yml` workflow runs `docfx docfx.json -o _site`
on every push to `main` and uploads the result as a GitHub Pages artifact.

To configure Pages for a repository created from this template, set
**Settings → Pages → Source = GitHub Actions** once. After that, every push to
`main` republishes the site.
