## Issue #279 Symbolic Probabilistic Reasoning

Issue [#279](https://github.com/link-assistant/formal-ai/issues/279)
implements the R6 follow-up: probability evidence must be symbolic,
link-native, inspectable, deterministic, and separate from neural-network
inference.

| ID | Requirement | Status |
| --- | --- | --- |
| R237 | Probability evidence must be represented as append-only Links Notation records with provenance and caller-supplied timestamps. | Implemented by `ProbabilityEvidence`, `ProbabilitySourceProvenance`, and `ProbabilityStore::to_links_notation` in `src/probability.rs`. Covered by `probabilistic_evidence_is_link_native_append_only_and_replayable`. |
| R238 | Candidate ranking must change when new symbolic evidence is added, without changing neural weights or invoking neural inference. | Implemented by `rank_probability_candidates` and `select_formalization_candidate_with_probability_store`, which adjust symbolic candidate posterior scores from evidence weights only. Covered by `bayesian_symbolic_evidence_changes_candidate_ranking`. |
| R239 | The clarify-vs-guess policy must consume the probability margin between top candidates. | Implemented by feeding evidence-adjusted probabilities into the existing temperature selector; `policy:temperature_selection` records `margin=` and `epsilon=`. Covered by `probability_margin_feeds_clarify_vs_guess_policy`. |
| R240 | Markov-style transition evidence must be available for answer-candidate ranking. | Implemented by `ProbabilityModel::MarkovTransition` and the `markov_from` field in `ProbabilityRankingConfig`. Covered by `markov_transition_evidence_can_rank_answer_candidates`. |
| R241 | Offline mode must ignore live-only probability evidence while preserving cached-source provenance. | Implemented by `ProbabilityStore::target_weight`, `replay_into_event_log`, and `append_to_link_store` honoring cached source flags and emitting `policy:offline` for skipped live-only sources. Covered by `offline_mode_uses_cached_probability_sources_and_skips_live_only_sources`. |
| R242 | Probability evidence must be visible in traces and replayable into the durable link-store projection. | Implemented by `probability:evidence`, `probability:model`, `source:http`, and `cache_hit` event replay plus `append_to_link_store`. Covered by the issue #279 specification tests. |
