# Issue 835 requirements

This decomposition turns issue
[#835](https://github.com/link-assistant/formal-ai/issues/835) into independently
reviewable acceptance leaves.

| ID | Requirement | Smallest acceptance leaf |
| --- | --- | --- |
| R835-1 | Inspect files without claiming a universal legality oracle | `check_file_legality` always returns `not_provided` as the global verdict and emits an explicit no-global-verdict/not-legal-advice limitation |
| R835-2 | Assess the three requested legal-risk categories independently | Every requested jurisdiction receives separate national-security, forbidden-content, and copyright/IP assessments; missing evidence stays `unknown` |
| R835-3 | Make jurisdiction and policy provenance explicit | Arbitrary jurisdiction codes compose with versioned policy IDs, source URIs, trigger codes, confidence, evidence IDs, and required actions |
| R835-4 | Support real detector integrations without coupling their failures | `LegalityEvidenceProvider` adapters run independently; completed, failed, and fail-closed-skipped runs remain visible without one timeout erasing other results |
| R835-5 | Treat confirmed child-safety hash evidence as a strict safety boundary | Only an externally supplied authorized-provider receipt can confirm a prohibited match; that state suppresses hash/Exif derivatives, skips ordinary providers, refuses ordinary handling, and retains only safe provider references |
| R835-6 | Extract relevant embedded metadata with provenance | Exif author, copyright, camera, capture time, latitude, and longitude values carry field-level source locators |
| R835-7 | Generalize beyond images | Signature/extension classification and one report schema support image, document, audio, video, and other files |
| R835-8 | Expose the capability through library and CLI surfaces | The public Rust functions and `formal-ai file-legality` accept the same serializable configuration and emit the same structured report |
| R835-9 | Preserve research, tests, and self-hosting evidence | The case study retains raw GitHub captures, dated primary sources, the TDD contract, whole-pipeline tests, and one real Formal-AI/Agent-CLI-authored leaf |

The five contribution-policy leaves are: scope boundary, category independence,
jurisdiction/policy provenance, metadata/generalization, and provider safety.
The live Formal AI endpoint and real Agent CLI author the provider-safety leaf,
meeting the one-in-five floor. The other four leaves are manually authored and
do not carry self-authorship provenance.
