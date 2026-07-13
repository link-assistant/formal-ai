//! Role constants for tooling and capability surfaces: definition-merge,
//! natural-language tool access, feature-capability questions, Playwright,
//! comparison and research tables, summary classification, coding-catalog
//! aliases, and Wikidata entity and property anchors (issue #386).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*` (issue #386).

/// Semantic role: a request to merge definitions.
///
/// A word asking to merge or combine ("merge", "merged", "combine", "combined",
/// "fuse", "fusion", "объедин", "विलय", "合并"). Matched as a raw substring
/// through [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is
/// lower-cased. The definition-merge handler requires it together with
/// [`ROLE_DEFINITION_ARTIFACT_REQUEST`]. Carried by `definition_merge_action`;
/// read by the Rust definition-merge handler and its JS worker mirror.
pub const ROLE_DEFINITION_MERGE_ACTION: &str = "definition_merge_action";
/// Semantic role: a request for a definition, translation, or encyclopedia entry.
///
/// A word naming such an artifact ("definition", "definitions", "translation",
/// "translations", "translated", "wikipedia", "определени", "परिभाषा", "定义").
/// Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is lower-cased.
/// The definition-merge handler requires it together with
/// [`ROLE_DEFINITION_MERGE_ACTION`]. Carried by `definition_artifact_request`;
/// read by the Rust definition-merge handler and its JS worker mirror.
pub const ROLE_DEFINITION_ARTIFACT_REQUEST: &str = "definition_artifact_request";
/// Semantic role: a phrase introducing the term whose definitions are merged.
///
/// A prefix phrase such as "definitions of" or "translation for", carried as a
/// prefix word form whose text before the slot marker is the phrase to locate.
/// The definition-merge handler scans the forms in declaration order through
/// [`crate::seed::Lexicon::role_word_forms`], filters to the prefix slot, finds
/// the first whose prefix appears in the prompt, and takes the text after it as
/// the term. The English forms are ordered longest-first so specific phrases win.
/// Carried by `definition_merge_marker`; read by the Rust definition-merge
/// handler and its JS worker mirror.
pub const ROLE_DEFINITION_MERGE_MARKER: &str = "definition_merge_marker";
/// Semantic role: a word that ends the term in a definition-merge prompt.
///
/// A boundary word such as "from", "using", "with", "by", "into", or "across".
/// Read through [`crate::seed::Lexicon::words_for_role`], reconstructed as a
/// space-padded token, and used to cut the extracted term at the earliest
/// boundary so the trailing source or method is trimmed away. Carried by
/// `definition_merge_tail_boundary`; read by the Rust definition-merge handler
/// and its JS worker mirror.
pub const ROLE_DEFINITION_MERGE_TAIL_BOUNDARY: &str = "definition_merge_tail_boundary";
/// Semantic role: a cue that the prompt is an explicit tool or API call.
///
/// The verbs "call", "invoke", "run" and the nouns "api", "tool" (plus their
/// translations). Read through [`crate::seed::Lexicon::mentions_role`] as whole
/// tokens; the natural-language-tool handler requires one of these together with
/// a named tool ([`ROLE_CALCULATOR_TOOL_NAME`] or [`ROLE_WEB_SEARCH_TOOL_NAME`])
/// before it treats the prompt as a direct tool call. Carried by
/// `tool_invocation_cue`; read by the Rust natural-language-tool handler.
pub const ROLE_TOOL_INVOCATION_CUE: &str = "tool_invocation_cue";
/// Semantic role: a surface word that names the calculator tool.
///
/// The English literal identifier `calculator` plus best-effort translations.
/// Read through [`crate::seed::Lexicon::mentions_role`] as a whole token; with a
/// co-occurring [`ROLE_TOOL_INVOCATION_CUE`] the natural-language-tool handler
/// routes the prompt to the `tool:calculator` capability. Carried by
/// `calculator_tool`; read by the Rust natural-language-tool handler.
pub const ROLE_CALCULATOR_TOOL_NAME: &str = "calculator_tool_name";
/// Semantic role: a surface word that names the web-search tool.
///
/// The English literal identifier `web_search` with its spaced and hyphenated
/// spellings, plus best-effort translations. Read through
/// [`crate::seed::Lexicon::mentions_role`] as a whole token; with a co-occurring
/// [`ROLE_TOOL_INVOCATION_CUE`] the natural-language-tool handler routes the
/// prompt to the `tool:web_search` capability. Carried by `web_search_tool`;
/// read by the Rust natural-language-tool handler.
pub const ROLE_WEB_SEARCH_TOOL_NAME: &str = "web_search_tool_name";
/// Semantic role: an explicit request to invoke the `local_shell` tool.
///
/// Whole phrases that bundle the verb and the tool name (`local_shell`, "local
/// shell tool", "invoke the shell tool", plus translations). Read through
/// [`crate::seed::Lexicon::mentions_role`] as whole tokens; decisive on its own,
/// so the handler does not also require a [`ROLE_TOOL_INVOCATION_CUE`] before it
/// routes the prompt to the `tool:local_shell` capability. Carried by
/// `local_shell_tool`; read by the Rust natural-language-tool handler.
pub const ROLE_LOCAL_SHELL_REQUEST_CUE: &str = "local_shell_request_cue";
/// Semantic role: a phrase introducing the argument of a tool call.
///
/// A marker such as "with query", "query", "with", or "for", carried in priority
/// order (longest first). When the argument is not delimited by backticks or
/// quotes, the handler reads the English forms through
/// [`crate::seed::Lexicon::words_for_role_in_languages`], reconstructs each as a
/// space-padded token, finds the first present in declaration order, and takes
/// the text after it as the argument. The non-English forms stay in the seed for
/// self-description. Carried by `tool_argument_marker`; read by the Rust
/// natural-language-tool handler.
pub const ROLE_TOOL_ARGUMENT_MARKER: &str = "tool_argument_marker";
/// Semantic role: a verb that commands the creation or modification of a file
/// (issue #680).
///
/// A write action word ("create", "write", "save", "make", "generate",
/// "append", plus translations), carried by `file_write_action` in
/// `data/seed/meanings-file-write.lino`. Read as whole tokens through
/// [`crate::seed::Lexicon::role_word_forms`] by the deterministic general change
/// planner ([`crate::agentic_coding::general_planner`]): it evidences a
/// file-write *intent* independently of any pinned phrasing, so a request in any
/// supported language routes to the advertised write tool instead of a prose
/// description.
pub const ROLE_FILE_WRITE_ACTION_CUE: &str = "file_write_action_cue";
/// Semantic role: a marker phrase that introduces the literal content of a file
/// to be written (issue #680).
///
/// A multi-word lead such as "containing", "with content", "with the following",
/// or "that says" (plus translations), carried as [`crate::seed::Slot::Prefix`]
/// forms by `file_write_content` in `data/seed/meanings-file-write.lino`. The
/// general change planner reads the literal before the slot through
/// [`crate::seed::Lexicon::role_word_forms`] and takes the text after it as the
/// payload. The bare words "content", "text", and "with" are deliberately absent
/// so a *read* request ("show me the contents of …") is never mistaken for a
/// write.
pub const ROLE_FILE_WRITE_CONTENT_LEAD: &str = "file_write_content_lead";
/// Semantic role: a word that introduces or names the target file of a write
/// (issue #680).
///
/// A naming or positional introducer ("file", "called", "named", "at", "as",
/// "in", "inside", plus translations), carried by `file_write_target` in
/// `data/seed/meanings-file-write.lino`. The general change planner accepts a
/// file-looking token as the write target only when it directly follows one of
/// these cues (or a [`ROLE_FILE_WRITE_DESTINATION_CUE`]), so an incidental dotted
/// token is not treated as a path.
pub const ROLE_FILE_WRITE_TARGET_CUE: &str = "file_write_target_cue";
/// Semantic role: a directional word that routes written content into a file
/// (issue #680).
///
/// A destination preposition ("to", "into", "onto", plus translations), carried
/// by `file_write_destination` in `data/seed/meanings-file-write.lino`. The
/// general change planner reads these through
/// [`crate::seed::Lexicon::role_word_forms`] to recognise the
/// "write CONTENT to FILE" shape, where the content precedes the file: only a
/// destination cue (not a positional [`ROLE_FILE_WRITE_TARGET_CUE`]) licenses
/// taking the span before the file as the payload, which keeps
/// "make sense of the file X" out of the write path.
pub const ROLE_FILE_WRITE_DESTINATION_CUE: &str = "file_write_destination_cue";
/// Semantic role: a surface alias naming a runtime feature capability.
///
/// Carried by the sixteen `feature_capability_*` meanings (for `web_search`,
/// `diagnostics`, `agent_mode`, and so on), each lexicalising the multilingual
/// phrases that name one feature. The feature-capability handler walks every
/// meaning carrying this role in declaration order — declaration order is
/// detection priority — and through
/// [`crate::seed::Lexicon::first_role_match_in_languages_raw`] returns the first
/// whose forms (in the prompt's language or English) occur as a raw substring.
/// The matched meaning's slug, minus its `feature_capability_` prefix, keys the
/// response table. Read by the Rust feature-capability handler and its JS worker
/// mirror.
pub const ROLE_FEATURE_CAPABILITY_ALIAS: &str = "feature_capability_alias";
/// Semantic role: an interrogative cue that flags a capability question.
///
/// Carried by the `feature_capability_question` meaning, whose per-language
/// lexemes hold cues such as "can you", "можешь", "能", and "क्या". The handler
/// checks them through
/// [`crate::seed::Lexicon::mentions_role_in_languages_raw`] against the prompt's
/// own detected language (English prompts additionally accept a grammatical
/// "is/are … enabled/available" frame computed in code) before it looks for a
/// named feature. Read by the Rust feature-capability handler and its JS worker
/// mirror.
pub const ROLE_FEATURE_CAPABILITY_QUESTION: &str = "feature_capability_question";
/// Semantic role: the GitHub repository-hosting platform.
///
/// Carried by `github_repository_platform`, with localized surface forms for
/// GitHub. The GitHub repository-traffic handler composes this with repository,
/// traffic, and question roles so visitor-visibility prompts are recognized by
/// meaning rather than by a hardcoded sentence.
pub const ROLE_GITHUB_REPOSITORY_PLATFORM: &str = "github_repository_platform";
/// Semantic role: a repository noun or abbreviation.
///
/// Carried by `repository_reference`. The GitHub repository-traffic handler
/// requires this together with the GitHub platform and traffic roles to keep
/// generic website-traffic prompts out of the repository-specific answer path.
pub const ROLE_REPOSITORY_REFERENCE: &str = "repository_reference";
/// Semantic role: repository traffic and visitor signals.
///
/// Carried by `github_repository_traffic_signal`, covering views, visitors,
/// visits, clones, referrers, and localized stems. Read by the Rust handler and
/// JS worker mirror for issue #497.
pub const ROLE_GITHUB_REPOSITORY_TRAFFIC_SIGNAL: &str = "github_repository_traffic_signal";
/// Semantic role: interrogative frames for GitHub traffic visibility.
///
/// Carried by `github_repository_traffic_question`, with prompts such as
/// "can I know", "who visited", "можно ли", "क्या", and "能知道". The handler
/// uses this as the question dimension of the platform∧repository∧traffic
/// recognition.
pub const ROLE_GITHUB_REPOSITORY_TRAFFIC_QUESTION: &str = "github_repository_traffic_question";
/// Semantic role: an action frame asking the assistant to perform arithmetic.
///
/// Carried by the `feature_action_arithmetic` meaning. When a capability
/// question also opens with one of these English frames ("can you calculate",
/// "can you compute"), reconstructed as a space-padded prefix, the handler steps
/// aside so the arithmetic solver answers instead of reporting availability.
/// Only the English frames drive the gate; the other languages stay in the seed
/// for self-description. Read by the Rust feature-capability handler and its JS
/// worker mirror.
pub const ROLE_FEATURE_ACTION_ARITHMETIC: &str = "feature_action_arithmetic";
/// Semantic role: an action frame asking the assistant to perform a planning task.
///
/// Carried by the `feature_action_planning` meaning. When a capability question
/// also contains one of these English frames ("can you summarize", "can you
/// brainstorm", "can you roleplay"), reconstructed as a space-padded token, the
/// handler steps aside so the primary planning handler answers. Only the English
/// frames drive the gate; the other languages stay in the seed for
/// self-description. Read by the Rust feature-capability handler and its JS
/// worker mirror.
pub const ROLE_FEATURE_ACTION_PLANNING: &str = "feature_action_planning";
/// Semantic role: the proper noun naming the Playwright automation tool.
///
/// Carried by the `playwright` meaning. The playwright-script handler asks
/// whether the tool is named by checking this role through
/// [`crate::seed::Lexicon::mentions_role_raw`] (a raw substring across every
/// language), so the proper noun and its common 'playright' misspelling live in
/// the data. The misspelling form carries an `action` naming the canonical
/// spelling; the handler walks [`crate::seed::Lexicon::role_word_forms`] for a
/// form whose action is set and occurs in the prompt to report the spelling
/// correction — the typo and its fix are data, not literals in the code. Read by
/// the Rust playwright-script handler and its JS worker mirror.
pub const ROLE_PLAYWRIGHT_TOOL_NAME: &str = "playwright_tool_name";
/// Semantic role: a cue that a Playwright prompt is requesting a script.
///
/// Carried by the `playwright_script_request_cue` meaning, whose per-language
/// lexemes hold the artifact nouns (script, test, spec, code) and authoring
/// frames (write, create, generate, make, build, can you, could you, and their
/// other-language equivalents). The playwright-script handler routes only when a
/// `playwright_tool_name` and one of these cues both occur, each checked through
/// [`crate::seed::Lexicon::mentions_role_raw`]. Read by the Rust playwright-script
/// handler and its JS worker mirror.
pub const ROLE_PLAYWRIGHT_SCRIPT_CUE: &str = "playwright_script_cue";
/// Semantic role: a strong trigger requesting a comparison table.
///
/// Carried by the `compare` meaning, whose lexemes hold the phrase 'comparison
/// table' and the verbs 'compare'/'comparing' (and their translations). The
/// research comparison-table handler opens when this role is mentioned
/// token-bounded via [`crate::seed::Lexicon::mentions_role`]; a match alone
/// satisfies the gate. Read by the Rust research-table handler and its JS worker
/// mirror.
pub const ROLE_COMPARISON_TABLE_TRIGGER: &str = "comparison_table_trigger";
/// Semantic role: the bare 'table' noun — the weak arm of the comparison gate.
///
/// Carried by the `table` meaning. On its own a table noun is too weak to open
/// the comparison-table handler; it counts only when it co-occurs with a
/// `comparison_difference_cue`, both checked token-bounded through
/// [`crate::seed::Lexicon::mentions_role`]. Read by the Rust research-table
/// handler and its JS worker mirror.
pub const ROLE_COMPARISON_TABLE_NOUN: &str = "comparison_table_noun";
/// Semantic role: a 'differences' cue — the partner of the bare table noun.
///
/// Carried by the `differences` meaning. When a `comparison_table_noun` and this
/// cue both occur (each checked token-bounded via
/// [`crate::seed::Lexicon::mentions_role`]) the weak arm of the comparison-table
/// gate opens. Read by the Rust research-table handler and its JS worker mirror.
pub const ROLE_COMPARISON_DIFFERENCE_CUE: &str = "comparison_difference_cue";
/// Semantic role: a signal that an earlier turn was a research request.
///
/// Carried by the `research_prompt_signal` meaning, mixing prefix surfaces
/// (`search …`, `find information …`, `look up information …` — matched when the
/// prompt starts with the literal before the `…` slot) and bare markers (`search
/// for information`, `web search`, `research` — matched token-bounded). The
/// research comparison-table handler reuses the prior research prompt for its
/// topics, reading the bare markers through
/// [`crate::seed::Lexicon::mentions_role`] and the prefix surfaces through
/// [`crate::seed::Lexicon::role_word_forms`] filtered to [`crate::seed::Slot::Prefix`]
/// then matched against the prompt with `starts_with`. Read by the Rust
/// research-table handler and its JS worker mirror.
pub const ROLE_RESEARCH_PROMPT_SIGNAL: &str = "research_prompt_signal";
/// Semantic role: a comparison-table column criterion.
///
/// Carried by the four criterion meanings `key_differences`, `use_cases`,
/// `advantages`, and `disadvantages`, in that declaration order — declaration
/// order is column order. The research comparison-table handler walks the
/// meanings carrying this role through
/// [`crate::seed::Lexicon::meanings_with_role`] and, for each text fragment, adds
/// the criterion when any of its surface words occurs as a raw substring; the
/// matched meaning's slug keys the column. The English triggers (including the
/// space-guarded `pro ` and ` con `) live in the data. Read by the Rust
/// research-table handler and its JS worker mirror.
pub const ROLE_RESEARCH_CRITERION: &str = "research_criterion";
/// Semantic role: a cue that classifies a prose sentence during summarization.
///
/// Carried by the seven `summary_kind_*` leaf meanings (`summary_kind_install`,
/// `summary_kind_example`, `summary_kind_language`, `summary_kind_stars`,
/// `summary_kind_purpose`, `summary_kind_use_case`, `summary_kind_feature`) in
/// that declaration order, each a kind of the structural `summary_statement_kind`
/// genus. The project summarizer walks the meanings carrying this role through
/// [`crate::seed::Lexicon::meanings_with_role`] and classifies a lowercased
/// sentence as the first meaning whose surface fragments occur in it as a raw
/// substring, mapping the matched slug to a `StatementKind`; the `language` kind
/// additionally requires at most twelve whitespace words. Read only by the Rust
/// summarization classifier (there is no JS worker mirror of that pipeline).
pub const ROLE_SUMMARY_CLASSIFICATION_CUE: &str = "summary_classification_cue";
/// Semantic role: a surface that names one coding-catalog target language.
///
/// Carried by the ten `program_language_<slug>` leaf meanings (rust, python,
/// javascript, typescript, go, c, cpp, java, csharp, ruby), each a kind of the
/// structural `program_language` genus and, where a canonical concept exists,
/// also `defined_by` it (`program_language_rust` → `language_rust`, …). The
/// coding catalog's `program_language_by_alias` walks `PROGRAM_LANGUAGES` in
/// priority order and, for each, reads the surfaces of the meaning named
/// `program_language_<slug>` through [`crate::seed::Lexicon::meaning`], matching
/// a prompt token against them; the inline alias list it replaced is gone. A
/// coverage guard asserts every catalog language slug owns a meaning carrying
/// this role. Read by the Rust catalog matcher and its JS worker mirror
/// (`programLanguageFromPrompt`).
pub const ROLE_PROGRAM_LANGUAGE_ALIAS: &str = "program_language_alias";
/// Semantic role: a surface that names one coding-catalog target task.
///
/// Carried by the eleven `program_task_<slug>` leaf meanings (`hello_world`,
/// `count_to_three`, `list_files`, `list_files_arg`, `list_files_reverse_sort`,
/// `list_files_arg_reverse_sort`, `fizzbuzz`, `factorial`, `reverse_string`,
/// `sum_to_ten`, `fibonacci`), each a kind of the structural `program_task` genus
/// and, where a canonical archetype exists, also `defined_by` it
/// (`program_task_hello_world` → `hello_world`). The catalog's
/// `program_task_by_alias` walks `PROGRAM_TASKS` in priority order and, for
/// each, reads the surfaces of the meaning named `program_task_<slug>` through
/// [`crate::seed::Lexicon::meaning`], matching a prompt phrase against them; the
/// inline alias list it replaced is gone. A coverage guard asserts every catalog
/// task slug owns a meaning carrying this role. Read by the Rust catalog matcher
/// and its JS worker mirror (`programTaskFromPrompt`).
pub const ROLE_PROGRAM_TASK_ALIAS: &str = "program_task_alias";
/// Semantic role: a surface that names one Wikidata item (entity or class).
///
/// Carried by the nine `wikidata_item_<slug>` meanings in
/// `data/seed/meanings-wikidata.lino` (`apple`, `fruit`, `sorting_algorithm`,
/// `water`, `bread`, `carrot`, `wikidata`, `wikipedia`, `wiktionary`). Each such
/// meaning records the language-independent Wikidata Q-id in its `wikidata`
/// field and lists every multilingual surface that resolves to it. The
/// formalization item-anchor resolver
/// ([`crate::translation::formalization`]) walks the meanings carrying this role
/// through [`crate::seed::Lexicon::meanings_with_role`], reading the Q-id and the
/// canonical English label rather than the former hardcoded `ITEM_LABELS`
/// table. A coverage guard asserts the table the resolver used to embed is fully
/// reproduced by these meanings.
pub const ROLE_WIKIDATA_ENTITY_ANCHOR: &str = "wikidata_entity_anchor";
/// Semantic role: a surface that names one Wikidata binary-relation property.
///
/// Carried by the seven `wikidata_property_<slug>` meanings in
/// `data/seed/meanings-wikidata.lino` that drive subject-predicate-object
/// extraction (`subclass_of`, `instance_of`, `part_of`, `has_part`, `capital`,
/// `named_after`, `item_for_this_sense`). Each records the language-independent
/// Wikidata P-id in its `wikidata` field and lists every multilingual phrase
/// that signals the relation; the canonical English label is the first English
/// word. The binary-relation parser
/// ([`crate::translation::formalization`]) iterates the meanings carrying this
/// role in declaration order through [`crate::seed::Lexicon::meanings_with_role`]
/// instead of the former hardcoded `PROPERTY_PATTERNS` slice. A word form whose
/// `action` names another property meaning (for example `is a` →
/// `wikidata_property_subclass_of`) declares the ambiguous alternative the parser
/// offers alongside the primary reading.
pub const ROLE_BINARY_RELATION_PROPERTY: &str = "binary_relation_property";
/// Semantic role: the Wikidata property a translation action prompt resolves to.
///
/// Carried by the single `wikidata_property_translation` meaning in
/// `data/seed/meanings-wikidata.lino`, which records Wikidata property P5972 and
/// its multilingual surfaces. The translation branch of the formalizer
/// ([`crate::translation::formalization`]) reads the lone meaning carrying this
/// role through [`crate::seed::Lexicon::meanings_with_role`] to anchor the
/// predicate, replacing the former hardcoded `translation_predicate` lookup keyed
/// on the literal id `P5972`. Kept distinct from
/// [`ROLE_BINARY_RELATION_PROPERTY`] because the binary-relation parser skips the
/// translation property, which is handled by its own action branch.
pub const ROLE_TRANSLATION_PROPERTY: &str = "translation_property";
