# Issue 1584 requirement-to-evidence inventory

Source: [complete issue and acceptance criteria](https://github.com/link-assistant/formal-ai/issues/1584).
Reviewed issue and PR feedback adds no required sub-issues. A01–A07 refer to the seven
acceptance checkboxes in order. Every checkbox remains pending. Each row below
retains a distinct prose requirement, including the named standards and runtime surfaces.

| Acceptance | Requirement | Current implementation or verification | Remaining scope |
| --- | --- | --- | --- |
| A01 | Accept arbitrary free-form prose, incomplete thoughts, slang and typos | Opaque retention in `formalizeOutput`; JS tests | General intent understanding remains unverified |
| A01 | Accept strict CNL input without an input grammar wall | Output-only option validation; opaque input tests | Equivalence across all input dialects remains unverified |
| A01 | Map everyday and CNL variants to identical internal Links Notation | One declared proposition with two forms in five languages | The lexical identity is not the complete solver intent graph |
| A02 | Default to cnl-default in every supported language | Shared configuration defaults; five localized policy records | No complete multilingual intersection renderer |
| A02 | Use the common ASD-STE100, ACE, Plain Language and Minimal English rules | Profile lists required checks | Independent validators and compatible realization rules are missing |
| A02 | Prefer simple high-frequency words with one canonical meaning | Required unambiguous-vocabulary check | No authoritative frequency corpus or general sense selection |
| A02 | Use short sentences | Sentence-length negative check | Splitting arbitrary long meanings safely is unfinished |
| A02 | Use active voice and explicit subject-verb-object structure | Named active-voice check | Grammar realization and independent syntax checks are unfinished |
| A02 | Remove ambiguous pronouns, idioms and filler | Opaque text remains unchanged | Safe resolution and meaning-preserving rewriting are unfinished |
| A02 | Support English, Russian, Hindi, Chinese and localized equivalents | Five-language lexical corpus including Spanish | Localized standard grammars and broad language behavior are unfinished |
| A03 | Strict asd-ste100 output | Profile selection and sentence-length check | Approved vocabulary, authorized meanings and complete syntax are missing |
| A03 | Strict ACE output mappable to first-order logic | Profile selection; ace_grammar stays unchecked | ACE parser, renderer and semantic mapping are missing |
| A03 | Strict Gellish relation output | Profile selection | Relation dictionary and typed triple realization are missing |
| A03 | Strict SBVR business-rule output | Profile selection | Rule semantics and structured realization are missing |
| A03 | Plain Language ISO 24495-1 output | Profile selection | Reader-oriented evaluation and compliant realization are missing |
| A03 | Minimal English and localized NSM-based equivalents | Profile selection; small lexical policy | General NSM decomposition and localized syntax are missing |
| A03 | E-Prime without any form of to be | Finite negative filter; false-certificate and density-combination regressions | Complete English checker and compliant general rewriting are missing |
| A03 | Basic English controlled vocabulary | Profile selection | Authoritative vocabulary, inflection policy and decomposition are missing |
| A03 | Special English controlled vocabulary | Profile selection | Authoritative vocabulary and decomposition are missing |
| A03 | Globish controlled vocabulary | Profile selection | Authoritative vocabulary and decomposition are missing |
| A03 | NSM primes only, with syntax frames | Finite vocabulary negatives; syntax and semantics stay unchecked | Complete official inventories, polysemy handling and decomposition are missing |
| A03 | top-100, top-200, top-500 and top-1000 output | Validated tier selection | Corpus-backed ranks and meaning-preserving decomposition are missing |
| A03 | Arbitrary positive top-N output | Safe-integer tier validation; missing evidence remains unchecked | Strict top-1 conflicts with universal meaning preservation; policy unresolved |
| A03 | Everyday standard vocabulary | Independent density configuration | This setting alone does not satisfy a selected strict standard |
| A03 | dense and max-precision aliases | Alias normalization; declared alternative selection | Global minimum and exact lexical semantics are unverified |
| A03 | Global lino-arguments and CLI selection | Native and JS CLI flags plus environment defaults | Dedicated lino-arguments coverage remains unfinished |
| A03 | HTTP API request parameters | CLI/API parser tests; Chat Completions tests cover JSON, exact labels, streaming notices and tool call usage | Every protocol path is not covered |
| A03 | Desktop and browser UI settings | Persisted standard and density controls; browser screenshots | Full desktop/native runtime behavior remains unverified |
| A03 | WebAssembly worker settings | Generated worker projection and settings plumbing | WASM execution and behavioral parity remain unverified |
| A03 | Apply policy through formalize, summarize and deformalize | Output adapter retains source and a lexical graph | Complete pipeline and summarization integration are unfinished |
| A04 | Use existing exact rare, archaic or technical words | One declared omniscience equivalence | General lexical sense evidence and decomposition are missing |
| A04 | Minimize the number of words while preserving full meaning | Stable minimum over declared alternatives | Global minimality over existing words remains unchecked |
| A04 | Notify max-precision users and recommend dictionary lookup | Localized notice kept outside the retained lexical identity | Notice exists; whole max-precision acceptance remains pending |
| A05 | Explicit requests for ACE, ASD-STE100 and E-Prime | Whole-directive and delimited-prefix parsing | Selecting metadata is not strict realization |
| A05 | Simplify, use semantic primes, or use the top 200 words | Localized rewrite directives and previous-answer selection | General semantic decomposition is unfinished |
| A05 | Compress to the fewest exact words or max-precision | Localized compression directives | General exact compression and global optimality are unfinished |
| A05 | Switch standards and density independently during dialogue | History-aware user directives; option and HTTP tests | Complete dialogue and protocol coverage is unfinished |
| A05 | Preserve meaning through repeated simple and dense paraphrases | Declared lexical round trips; unknown bytes retained | General semantic equivalence and original intent scope are unverified |
| A05 | Preserve deterministic behavior | Stable ordering and tie breaks in the finite corpus | Determinism of the full requested system is unverified |
| A06 | Detect multiple plausible intent graphs from unrestricted input | Existing ambiguity handling was inspected | General candidate generation remains unfinished |
| A06 | Restate each candidate more precisely in CNL | No new candidate renderer | Candidate graph realization remains unfinished |
| A06 | Ask for confirmation before acting on ambiguous prompts | No new cross-runtime confirmation flow | Action gating for all candidate interpretations remains unfinished |
| A07 | Corpus includes free-form, colloquial and stream-of-consciousness styles | Opaque adversarial examples | Canonical intent corpus across all these styles is missing |
| A07 | Corpus includes primes, STE, ACE, Gellish, SBVR and plain variants | Named profiles exist | Independent input-standard fixtures and validators are missing |
| A07 | Corpus includes Basic English, E-Prime and technical dense variants | Two declared realizations per language | Broad independently validated corpus is missing |
| A07 | Every input style paired with every output standard and tier | 880 bounded lexical projections | This is a single-proposition output-profile matrix, not the requested matrix |
| A07 | Assert exact internal intent/meaning graph equality | Declared lexical identity comparison | Complete internal intent graph equivalence is unverified |
| A07 | Independently verify valid deterministic output for any input | Explicit unchecked report; new capture has zero certificates | Strict output compliance remains unfinished |
| A07 | Round trip free-form to Lino to dense to primes to cnl-default | Finite declared lexical round trips | Zero general semantic drift is unverified |
| A07 | Rust unit/integration and JS/WASM parity matrix | JS tests; native assertions; CI native type-and-lint | Native behavior, WASM execution and full parity are unverified |

The shared source is [cnl_output.mjs](../../js/agentic/crate/cnl_output.mjs), with
[policy data](../../data/seed/cnl-output.lino), a supplied [native port](../../rust/src/cnl_output.rs),
[JavaScript regressions](../../rust/tests/web/cnl-output.test.mjs),
[HTTP checks](../../rust/tests/web/cnl-output-http.test.mjs), and [native assertions](../../rust/tests/unit/cnl_output.rs).
The [reproduction and provenance record](README.md) describes the bounded corpus,
and the [PR case study](../../docs/case-studies/pull-request-1585/README.md) retains CI evidence.

The top-1 conflict is a requirement conflict; it is not evidence that the other
unfinished rows have been implemented. No exception, lossy fallback or vocabulary
widening has been approved. Missing implementation and missing behavioral evidence
remain unfinished. Type checking does not execute the native or WASM matrix.
