# Issue #1085 requirements

Source: the issue body (`raw-data/github/issue-1085.json`), sections 3 and 6, plus
the remaining failures of #1081 that konard asked to land in the same pull request
(`raw-data/github/issue-1081.json`). IDs match `REQUIREMENTS.md`.

| ID | Item | Requirement | Acceptance |
| --- | --- | --- | --- |
| R1085-1 | D1.4 | Kernel allowlist; five measured ceilings outside it; down-only; release shows the line ceiling lower than at the previous tag | gate fails on any measured value above its ceiling or any raised ceiling; status workflow fails when the line ceiling did not fall since the last tag |
| R1085-2 | D1.1-D1.3 | Seed loaded into the doublets store at startup; matching by link query; one generic interpreter; migrate pending handlers smallest-first | rule interpreter delivered, pending 40; store loading next |
| R1085-3 | D2 | Edits as link substitutions over the CST network, rendered, formatted, compiled, tested; requirement-shaped edits resolved by link query | each ladder leaf reproduced by a committed rule and `cargo check`; one prompt with no file name resolved to the right file |
| R1085-4 | D3.1 | Attribution by model | a commit with Claude-named evidence is not attributed; a missing `Formal-AI-Model` is a strict-gate error |
| R1085-5 | D3.2 | Behaviour-only paths | replay shows no numerator path under `docs/` or `dev/` |
| R1085-6 | D3.3 | Pull-request author in the row | row carries `<url> <login>` or `unrecorded` |
| R1085-7 | D3.4 | History restated by appending version-3 rows | `--replay-epoch` appends one row per earlier tag; nothing rewritten |
| R1085-8 | D3.5 | Floor off the release path; red-until-true status | `release.yml` has no gate; status workflow runs the floor on push and schedule |
| R1085-9 | D4 | Leaves compile and test; composites merge; depth >= 3 requirement-shaped; root is a real issue | `verify-node.sh` fails an uncompilable leaf |
| R1085-10 | D5.4 | Upstream row beside every curated citation | `VISION.md`, `ROADMAP.md` |
| R1085-11 | D5.1-D5.3 | Upstream failures feed the learning cycle; red on regression; HumanEval task 0 explained | delivered: frontier record, stagnation warning, transfer fixed |
| R1085-12 | #1081 | crates.io probe never uses `/me`; read-only verdict is `unknown` | test asserts `/api/v1/me` is not called and the token is not sent |
| R1085-13 | #1081 | macOS archive budget from measured durations | budget sum <= 70% of the cap |
| R1085-14..17 | D6-D9 | Frontier queue, evidence store, gate collapse, traceability column | sub-issues of #1085 |
