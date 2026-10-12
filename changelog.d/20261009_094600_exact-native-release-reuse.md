---
bump: patch
---

### Fixed

- Build native release targets once from one selected source and compiler contract, then verify executable and extracted CLI archive bytes before desktop and CLI packaging.
- Capture coherent release helpers from the workflow commit so older selected tags remain buildable; heal missing version-specific VSIX and checksum/provenance manifests.
- Prepare the default-feature test-profile library before the unchanged timed doctest step; bind full Docker release metadata to the checked-out release commit.
- Correct native lint diagnostics and preserve exact gate identity and answer-field contracts. Native CI and publication verification remain required.
