# Online research notes (collected by coordinator 2026-11-10)

## GitHub issue mechanics
- Issue dependencies REST: GET/POST /repos/{o}/{r}/issues/{n}/dependencies/blocked_by (POST body {"issue_id": <numeric id>}), DELETE .../blocked_by/{issue_id}. https://docs.github.com/en/rest/issues/issue-dependencies
- Sub-issues: up to 100 sub-issues per parent, up to eight levels of nesting. https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues ; cap raised from 50 + REST API added: https://github.blog/changelog/2024-12-12-github-issues-projects-close-issue-as-a-duplicate-rest-api-for-sub-issues-and-more/
- Issue types GA note: https://github.blog/changelog/2025-04-08-evolving-github-issues-and-projects/

## Recursive problem solving
- HTN overview (compound tasks refined step-wise into primitive tasks until executable plan; method choice nondeterministic; HTN planning undecidable in general): https://arxiv.org/pdf/1403.7426
- Soar: universal subgoaling (any decision can be object of goal-oriented attention), impasse -> substate, chunking turns subgoal work into rules. Laird/Rosenbloom/Newell 1986 https://doi.org/10.1007/978-1-4613-2277-1_2 ; https://en.wikipedia.org/wiki/Soar_(cognitive_architecture)
- Parsel (Zelikman et al., NeurIPS 2023): decompose task into nested NL function descriptions, generate candidates per function, test combinations; APPS pass rates >75% higher than direct sampling; HumanEval pass@1 67%->85% with generated tests; decomposition alone (without compose+test search) does not give the gain. https://arxiv.org/abs/2212.10561

## Tests-first / candidate ranking
- CodeT (Chen et al., ICLR 2023): generate tests, execute candidates, score by dual agreement; HumanEval pass@1 65.8% (+18.8 abs). https://arxiv.org/abs/2207.10397
- AlphaCode clusters candidates by behaviour on generated inputs. Functional-overlap reranking https://arxiv.org/pdf/2311.03366 ; TDD for code generation https://www.arxiv.org/pdf/2408.13976

## Agent failure taxonomies
- "An Empirical Study on Failures in Automated Issue Solving" (2025): 9 categories / 25 subcategories over Localization, Repair, Iteration&Validation phases. https://arxiv.org/abs/2509.13941
- "Coherence Collapse" (2026): 16,758 trajectories; 60-69% of failures reach and edit correct functions yet produce wrong patch. https://www.alphaxiv.org/abs/2603.24631
- PAGENT https://arxiv.org/html/2506.17772v1 ; trajectory study https://arxiv.org/pdf/2511.00197v1

## Licences
- REUSE spec 3.3 (per-file SPDX-License-Identifier, LICENSES/ dir, LicenseRef- for non-SPDX): https://reuse.software/spec-3.3/ ; SPDX ids https://spdx.dev/ids ; handling https://spdx.dev/learn/handling-license-info/

## Probabilistic logic
- PLN vs NARS (Goertzel 2024): similar strength×confidence under highly uncertain term probabilities; NARS frequency/confidence; MLN/Bayes nets struggle with quantifiers/higher-order. https://arxiv.org/abs/2412.19524

## Formalization / naturalization
- Surface realization = producing text from formal meaning representation; Basile & Bos 2013 https://www.let.rug.nl/bos/pubs/BasileBos2013ENLG.pdf
- Grammatical Framework: abstract syntax shared, parse (analysis) and linearize (synthesis) are inverse; ACE in GF ported to 5 languages. Ranta et al. 2020 https://research.chalmers.se/publication/517996/file/517996_Fulltext.pdf ; ACE translates to FOL https://arxiv.org/pdf/1406.2903
- NASA FRET: FRETISH constrained NL -> temporal logic; https://arxiv.org/pdf/2503.24040

## relative-meta-logic (checked via gh 2026-11-10)
- link-foundation/relative-meta-logic: Unlicense, Rust primary, no GitHub release, pushed 2026-11-05; open issues #183 "Make sure it really meta", #185 publish crate + no_std probability core, #186 link-cli memory, #187 CI, #188 crates.io publish. Not a Cargo/npm dependency of formal-ai (rust/src/relative_meta_logic.rs is in-repo).
