### Changed

- Browser engine and VS Code packages follow automated releases, resolve only the matching trusted main commit, and attach installable archives to GitHub releases. Pull requests verify packaging without publishing.
- Repeated npm publication verifies the immutable tarball bytes; registry authentication and transport errors fail visibly.
- Automatic and manual releases publish the slim Docker image and verify its live chat and file-write tool responses before creating a GitHub release. Pull requests exercise the same runtime contract and image size limit.
- Platform CI is enabled for this pull request, including production desktop and CLI build checks.
