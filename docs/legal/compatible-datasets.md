# Candidate datasets for Formal AI source review

Reviewed on 2026-08-01. This is a dated shortlist of ten recent, relevant
candidates, not an approval registry and not a claim that every record is
compatible. The only authoritative approval state is
[`data/training/source-registry.json`](../../data/training/source-registry.json),
which currently contains no approved training or distillation sources.

“Candidate” means that a first-party card exposes enough license or provenance
information to begin a review. “Compatible” is earned only after an exact
revision, subset, and purpose passes [`source-review.md`](source-review.md).
Database-level licenses often do not license each contained work. Web crawl
availability is not permission, and synthetic text inherits questions about
its seeds and generator terms.

## Ten candidates

The order favors the newest upstream dataset generation or release described by
the cited maintainer, then breadth and relevance. It is not a performance
ranking or an exhaustive global “latest” list.

| # | Exact candidate | Upstream date/license signal | Compatibility boundary | Intake state |
| --- | --- | --- | --- | --- |
| 1 | [PleIAs SYNTH](https://huggingface.co/datasets/PleIAs/SYNTH), exact snapshot required | Collection updated 2025; card identifies Apache-2.0 and row-level `seed_license` metadata | Allowlist only rows whose seeds, generator route, and output terms are independently cleared; do not infer “copyright-free” from “synthetic” | Pending |
| 2 | [Common Pile v0.1 collections](https://huggingface.co/common-pile) | Released 2025; public-domain and openly licensed components | Select components and licenses individually; exclude attribution/share-alike components if the planned unqualified dedication cannot carry their terms; retain license-laundering warning | Pending |
| 3 | [FineWeb2](https://huggingface.co/datasets/HuggingFaceFW/fineweb-2), pinned language/crawl subset | 2025 paper; ODC-By-1.0 plus Common Crawl terms | ODC-By governs database rights, not necessarily each page; filter rights reservations, personal data, and incompatible content and retain attribution | Pending |
| 4 | [SmolTalk](https://huggingface.co/datasets/HuggingFaceTB/smoltalk), only named new synthetic subsets | Released 2025; maintainers mark newly generated subsets Apache-2.0 | The mixture includes reused datasets under their original licenses; approve each subset and generator provenance separately | Pending |
| 5 | [DCLM Baseline 1.0](https://huggingface.co/datasets/mlfoundations/dclm-baseline-1.0), immutable revision | 2024 DCLM release; project code and data card are public | Do not treat the repository's MIT code license as a content license; review the Common Crawl source, removal policy, and each intended shard | Pending |
| 6 | [FineWeb](https://huggingface.co/datasets/HuggingFaceFW/fineweb), pinned v1.4/crawl subset | 2024 generation; ODC-By-1.0 plus Common Crawl terms | Same database/content distinction as FineWeb2; page-level rights and privacy review remain necessary | Pending |
| 7 | [Dolma 1.7](https://huggingface.co/datasets/allenai/dolma), exact component and revision | 2024-04 release; ODC-By-1.0 | The card expressly binds users to original-source agreements; separately review web, code, papers, books, Reddit, and encyclopedia components | Pending |
| 8 | [The Stack v2](https://github.com/bigcode-project/the-stack-v2), exact source repositories/files | 2024 curation release; curation code is Apache-2.0 | Code files retain repository/file licenses and notices; honor opt-outs and exclude no-license or incompatible-license material; the curation-code license is not a payload license | Pending |
| 9 | [Tulu 3 SFT Mixture](https://huggingface.co/datasets/allenai/tulu-3-sft-mixture), allowlisted components only | 2024 mixture; collection marked ODC-By-1.0 | Component cards contain different licenses and third-party generated outputs; never approve the mixture as one homogeneous source | Pending |
| 10 | [RedPajama-Data-v2](https://github.com/togethercomputer/RedPajama-Data), pinned snapshot/subset | 2023 release with maintained 2024-era tooling; repository code is Apache-2.0 | Common Crawl documents retain underlying rights; the code license does not license payload text; apply URL, rights-reservation, privacy, and deletion filters | Pending |

The separate [Smol-SmolTalk card](https://huggingface.co/datasets/HuggingFaceTB/smol-smoltalk)
is useful corroboration for an Apache-2.0 synthetic subset, but it is not an
eleventh independent provenance family. Likewise, a hub license tag is an index
hint, not the legal evidence for a source-registry decision.

## Required filters before approval

- Pin an immutable revision and enumerate every selected subset and file.
- Record both the dataset/database license and underlying content licenses.
- Preserve attribution, notices, copyright/TDM reservations, opt-outs, and
  deletion mechanisms.
- Exclude secrets, leaked or access-controlled code, and all real personal or
  sensitive data.
- For synthetic rows, record seed rights, generator identity and route, and the
  provider's affirmative training and distribution permission.
- Test for duplicates, memorized long passages, license-marker loss, and
  contamination by benchmark answers.
- Define the intended parameter update and release license before acquisition;
  a later purpose change requires a new review.

No row above may be downloaded into `data/training/artifacts/` while its state is
`Pending`.
