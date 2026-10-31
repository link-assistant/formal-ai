### Changed
- Restrict production compilation and deployments to main and authorized release consumers; use JavaScript for lightweight PR admission and one group of selected unit tests.
- Avoid redundant Cargo package compilation and make independent Docker cold builds, browser E2E and box-language integration explicit main-only qualifications.
