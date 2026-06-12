---
bump: minor
---

### Added
- Added deterministic installation conversion support for README.md
  install/deploy guides, Bash/sh scripts, and PowerShell scripts. The new
  `installation_conversion` handler extracts ordered install commands into a
  shared IR, renders scripts or README guides from that IR, and is mirrored in
  the browser worker so conversion prompts no longer fall through to `unknown`
  or generic script generation.

- Added issue #423 regression coverage, including README-to-Bash/PowerShell,
  script-to-README, nested fenced README content, and a 50-project GitHub
  repository corpus captured from the most-starred repository snapshot.
