//! Resolve a user turn into the concrete shell command the agentic loop should run.
//!
//! Split out of [`super::planner`] (issue #676): the agentic planner used to know
//! only the hardcoded `ls`, so `execute pwd` and every other seed shell token fell
//! through to the *unknown* fallback. The two data-driven strategies here — a named
//! command backed by `data/seed/terminal-commands.lino`, and a natural-language
//! directory-listing request — make the whole seed vocabulary reachable.
//!
//! Sentence scoping and command-policy classification live next door in
//! [`super::shell_command_policy`], which keeps both files under the repository
//! line budget.

use super::shell_command_policy::{
    governs_commands_rather_than_requesting_one, is_prose_word, named_shell_command_in_sentence,
    normalize_command_word, sentence_spans, states_a_command_policy,
};
use super::directory_listing::asks_for_directory_listing;
use super::file_path_shape::{is_dotted_number, trim_trailing_sentence_dot};
use super::workspace_inspection::asks_about_the_workspace;
use crate::seed::{self, ShellIntentArgument, ShellIntentVocabulary, TerminalCommandVocabulary};

const REPORT_ISSUE_ACTION: &str = "formal-ai:report-issue";
/// Resolve a user turn into the concrete shell command the agentic loop should run.
///
/// Two data-driven strategies, in order of specificity:
///
/// 1. **Named command** ([`named_shell_command`]): when the prompt pairs a run/execute
///    verb (or terminal/shell phrase) with a known shell token from
///    `data/seed/terminal-commands.lino` — e.g. *"execute pwd"*, *"run git status"*,
///    *«запусти ls»* — emit that command (with its flag/path/sub-command arguments).
///    This is what makes `pwd` (issue #676) and every other seed token reachable, not
///    just the hardcoded `ls` the fallback used to know.
///
/// 2. **Natural-language directory listing** ([`asks_for_directory_listing`]): when the
///    prompt asks, in prose, to see the files in the current place — e.g. *"give me a
///    list of files in current folder"*, *"what files are here?"* — resolve to `ls`.
///
/// The vocabulary lives in seed data, so a maintainer retunes coverage by editing a
/// `.lino` file rather than this function, upholding the project rule against hardcoded
/// natural language in the solver.
pub(super) fn shell_command_for_task(prompt: &str) -> Option<String> {
    let prompt = strip_balanced_outer_quotes(prompt.trim());
    // Caller policy is filtered here, not inside one strategy, because
    // `prefixed_shell_command` below runs first and matches on the whole prompt:
    // a rule applied only to `named_shell_command` would leave
    // "Run commands with sudo only when necessary." passing straight through as
    // an explicitly introduced command (issue #907, follow-up).
    if governs_commands_rather_than_requesting_one(prompt) {
        return None;
    }
    let vocab = seed::terminal_command_vocabulary();
    let intent_vocab = seed::shell_intent_vocabulary();
    let intent = intent_shell_command(prompt, &intent_vocab);
    let listing = asks_for_directory_listing(prompt);
    let web_search = crate::solver_handlers::web_search_query_for(prompt).is_some();
    let semantic = listing.then(|| String::from("ls")).or(intent);

    if let Some(command) = prefixed_shell_command(prompt, &vocab) {
        let first = command
            .split_whitespace()
            .next()
            .map(normalize_command_word);
        let names_known_command = first
            .as_ref()
            .is_some_and(|first| vocab.shell_tokens.iter().any(|token| token == first));
        let names_semantic_command = semantic.as_ref().is_some_and(|semantic| {
            semantic
                .split_whitespace()
                .next()
                .map(normalize_command_word)
                == first
        });
        if names_semantic_command {
            return semantic;
        }
        if names_known_command || (semantic.is_none() && !web_search) {
            return Some(command);
        }
    }

    semantic
        .or_else(|| named_shell_command(prompt, &vocab))
        .or_else(|| bare_shell_command(prompt, &vocab))
}

/// Resolve only the seed-backed natural-language shell intent.
///
/// The interactive solver uses this narrower entry point after its explicit
/// terminal syntax checks, so ordinary Chat requests and the agentic planner
/// share one semantic intent table without broadening Chat detection to every
/// named shell token.
pub fn semantic_shell_command_for_task(prompt: &str) -> Option<String> {
    intent_shell_command(prompt, &seed::shell_intent_vocabulary())
}

fn strip_balanced_outer_quotes(prompt: &str) -> &str {
    for quote in ['"', '\'', '`'] {
        if prompt.starts_with(quote) && prompt.ends_with(quote) && prompt.len() >= 2 {
            return &prompt[1..prompt.len() - 1];
        }
    }
    prompt
}

/// Pass an explicitly introduced command through byte-for-byte (apart from
/// surrounding whitespace). Explicit execution is an intent boundary: the
/// command need not appear in a maintained binary allowlist because the client
/// still owns its normal sandbox and permission decision.
///
/// The remainder must still *look* like a command line rather than a sentence
/// about one. `run`/`execute` are ordinary English verbs, so "Run all tests in
/// the background" and "Execute nothing without asking the operator first"
/// reach this function exactly like "run cargo test" does — and passing their
/// prose through produced commands such as `all tests in the background`. The
/// allowlist stays absent: an unknown binary (`mytool --flag`, `./build.sh`) is
/// accepted, and only prose is refused.
fn prefixed_shell_command(prompt: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    let prompt = prompt.trim();
    let lower = prompt.to_lowercase();
    let prefix = vocab
        .passthrough_prefixes
        .iter()
        .filter(|prefix| prefix_boundary(&lower, prefix))
        .max_by_key(|prefix| prefix.chars().count())?;
    let remainder = prompt.get(prefix.len()..)?.trim_start();
    let remainder = remainder
        .strip_prefix(':')
        .unwrap_or(remainder)
        .trim_start();
    let remainder = strip_balanced_outer_quotes(remainder);
    if let Some(named) = command_named_in_prose(remainder, vocab) {
        return Some(named);
    }
    (!remainder.is_empty() && !reads_as_prose(remainder, vocab)).then(|| remainder.to_owned())
}

/// Characters that turn the words around them into data rather than prose:
/// quoting, redirection, substitution and command separators.
const SHELL_QUOTING_AND_METACHARACTERS: &[char] =
    &['"', '\'', '`', '|', '&', ';', '<', '>', '$', '(', ')', '{', '}'];

/// Recover the command from a remainder that *names* it instead of being it.
///
/// Issue #866 and #867 reported *"Execute ls command"* running `/bin/ls
/// command`, which fails with *cannot access 'command'*. The trailing noun
/// names the command; it is not an argument to it. Seed data declares those
/// nouns per language in `data/seed/terminal-commands.lino`, and their presence
/// is the tell that the remainder is a noun phrase *about* a command rather
/// than a command line — so the command is collected out of it, dropping the
/// determiners that lead the phrase and stopping at the prose that ends it,
/// exactly as argument collection does elsewhere.
///
/// Quoting and shell metacharacters switch the recovery off, because inside
/// them a word is data: `git commit -m 'fix command parsing'` keeps its message.
fn command_named_in_prose(remainder: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    if remainder.contains(|c| SHELL_QUOTING_AND_METACHARACTERS.contains(&c)) {
        return None;
    }
    let words = remainder.split_whitespace().collect::<Vec<_>>();
    let kept = words
        .iter()
        .copied()
        .filter(|word| !names_a_command(word, vocab))
        .collect::<Vec<_>>();
    if kept.len() == words.len() || kept.is_empty() {
        return None;
    }
    let start = kept.iter().position(|word| !is_prose_word(word))?;
    let mut command = vec![kept[start]];
    for word in &kept[start + 1..] {
        if is_prose_word(word) {
            break;
        }
        command.push(word);
    }
    Some(command.join(" "))
}

/// Whether `word` is a seed-declared noun that names a command. Punctuation is
/// trimmed and case folded across scripts, so *«команду»* and *"Command."* both
/// match their seed entry.
fn names_a_command(word: &str, vocab: &TerminalCommandVocabulary) -> bool {
    let normalized = word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    !normalized.is_empty() && vocab.command_nouns.iter().any(|noun| noun == &normalized)
}

/// Whether a recovered command line is really a sentence describing an action.
///
/// Two tells, either of which is enough: the command position holds a
/// natural-language word, or the arguments carry two or more function words. A
/// genuine command line spends its arguments on paths and flags, not on *in
/// the*, *without asking*, or *so that*.
///
/// A known shell token in command position settles it outright, because the two
/// vocabularies overlap: `file` is both a Unix command and an ordinary English
/// word, so `execute file Cargo.toml` must stay a command.
fn reads_as_prose(remainder: &str, vocab: &TerminalCommandVocabulary) -> bool {
    let mut words = remainder.split_whitespace();
    let Some(first) = words.next() else {
        return true;
    };
    // A web address is a resource to act on, not a program to run. Issue #862
    // asked to *"Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in
    // Rust"* — the task published at that address, not the address typed at a
    // shell — and the passthrough prefix happily handed the whole line over.
    if first.contains("://") {
        return true;
    }
    let command = normalize_command_word(first);
    if vocab.shell_tokens.iter().any(|token| token == &command) {
        return false;
    }
    is_prose_word(first) || words.filter(|word| is_prose_word(word)).count() >= 2
}

fn bare_shell_command(prompt: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    let first = prompt.split_whitespace().next()?;
    let command = normalize_command_word(first);
    // A bare command token can also be an ordinary imperative (notably
    // `find information about ...`).  Explicit passthrough prefixes above are
    // unambiguous, but a seed-backed web-search request must remain available
    // to the dedicated search router instead of becoming a shell command.
    if crate::solver_handlers::web_search_query_for(prompt).is_some() {
        return None;
    }
    let is_known = vocab
        .bare_shell_tokens
        .iter()
        .any(|token| token == &command);
    let words = prompt.split_whitespace().collect::<Vec<_>>();
    (is_known && words[1..].iter().all(|word| !is_prose_word(word))).then(|| prompt.to_owned())
}

fn prefix_boundary(prompt: &str, prefix: &str) -> bool {
    prompt.starts_with(prefix)
        && prompt
            .get(prefix.len()..)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|c| c.is_whitespace() || c == ':')
}

/// Recover the literal subject of a seed-backed source-code search request.
///
/// A client may advertise a dedicated grep/code-search tool instead of a shell.
/// The planner uses this semantic query before falling back to the `rg` lowering
/// returned by [`shell_command_for_task`].
///
/// The request has to spell the act out — "search the repository for
/// `task_decomposition`". A request that only says what the caller wants to
/// know is a different admission with a stricter subject rule; see
/// [`workspace_inspection_search_for_task`].
pub(super) fn code_search_query_for_task(prompt: &str) -> Option<String> {
    let vocab = seed::shell_intent_vocabulary();
    super::stated_request::request_blocks(prompt)
        .into_iter()
        .find_map(|block| {
            let lower = block.to_lowercase();
            let cue = longest_search_cue(&lower, &vocab)?;
            literal_code_search_query(&lower)
                .or_else(|| shaped_code_search_token(block))
                .or_else(|| adjacent_code_search_token(block, &lower))
                .or_else(|| local_search_query(block, &cue, &vocab))
        })
}

/// Recover the subject of a request that asks about the workspace itself.
///
/// "Inspect the existing task-decomposition data model and identify where a
/// node stores its children" never says *search*, but an agent that has been
/// handed a repository answers a question about that repository by reading it.
/// The planner resolves this ahead of the open-web routers, which would
/// otherwise claim the request on the strength of its question shape alone
/// (issue #1066).
///
/// What keeps the open web reachable is the subject, not the verb. The subject
/// must either be visibly code-shaped — quoted or carrying an underscore, dot,
/// interior capital, or hyphen — or sit next to a seed-declared source-artifact
/// kind. That is the difference between "verify the retry-policy helper" or
/// "verify the retry check" and "verify the current exchange rate for the
/// euro". The whole-prompt fallback [`code_search_query_for_task`] ends with is
/// deliberately absent here: it works by deleting an explicit cue, and this
/// admission has no cue to delete.
pub(super) struct WorkspaceInspectionSearch {
    /// The code-shaped subject used as the human-readable query.
    pub(super) query: String,
    /// The grep expression aimed at the fact requested about that subject.
    pub(super) pattern: String,
    /// A filename filter when the subject itself names a module-like file.
    pub(super) include: Option<String>,
}

/// Resolve a workspace question into a subject and a fact-focused search.
///
/// A module name tells the agent *where* an answer is likely to live, but it is
/// rarely the answer. Searching only for `task_decomposition` can fill Agent's
/// result cap with release notes before reaching the `children` field the
/// caller asked about. The request's remaining content words therefore form
/// the grep expression, while a lowercase underscored subject narrows the file
/// set. Both are recovered structurally; no ladder wording is registered here.
pub(super) fn workspace_inspection_search_for_task(
    prompt: &str,
) -> Option<WorkspaceInspectionSearch> {
    for block in super::stated_request::request_blocks(prompt) {
        if !asks_about_the_workspace(block) {
            continue;
        }
        // Agent's compactor may flatten the blank line between the actual
        // checkout question and a machine-shaped worker contract. Prefer the
        // subject carried by the sentence that asks to inspect the workspace;
        // otherwise a later token such as `new_audit_effect` can outrank the
        // helper the caller asked about. Keep the historical whole-block
        // fallback for requests whose inspection cue and subject span two
        // sentences.
        if let Some(search) = sentence_spans(block)
            .into_iter()
            .filter(|sentence| asks_about_the_workspace(sentence))
            .find_map(workspace_inspection_search)
        {
            return Some(search);
        }
        if let Some(search) = workspace_inspection_search(block) {
            return Some(search);
        }
    }
    None
}

fn workspace_inspection_search(text: &str) -> Option<WorkspaceInspectionSearch> {
    let query = code_shaped_query(text)?;
    let terms = inspection_fact_terms(text, &query);
    let canonical_fact = literal_inspection_fact_query(&text.to_lowercase())
        .or_else(|| serialized_relationship_fact_query(text));
    // A canonical fact expression identifies the implementation independently
    // of its module. An inferred code-shaped subject can name a subsystem (for
    // example `task-strategy`) whose implementation lives in a differently
    // named file, so it must not exclude the canonical source fact.
    let include = canonical_fact
        .as_ref()
        .map(|_| "src/**/*".to_owned())
        .or_else(|| inspection_filename_filter(text, &query));
    if let Some(pattern) = canonical_fact {
        return Some(WorkspaceInspectionSearch {
            query,
            pattern,
            include,
        });
    }
    let mut pattern_terms = Vec::new();
    if include.is_none() {
        pattern_terms.push(query.clone());
    }
    pattern_terms.extend(terms);
    let pattern = if pattern_terms.is_empty() {
        query.clone()
    } else {
        pattern_terms.join("|")
    };
    Some(WorkspaceInspectionSearch {
        query,
        pattern,
        include,
    })
}

/// Content words that describe the fact requested by a workspace inspection.
///
/// The seed-declared inspection actions and code-subject kinds express the
/// request's grammar rather than its answer, so they are excluded alongside
/// ordinary prose words. The remainder is useful both for constructing the
/// grep and for choosing the most relevant line from grouped grep output.
pub(super) fn workspace_inspection_terms_for_task(prompt: &str) -> Vec<String> {
    for block in super::stated_request::request_blocks(prompt) {
        if !asks_about_the_workspace(block) {
            continue;
        }
        if let Some((sentence, query)) = sentence_spans(block)
            .into_iter()
            .filter(|sentence| asks_about_the_workspace(sentence))
            .find_map(|sentence| code_shaped_query(sentence).map(|query| (sentence, query)))
        {
            return inspection_fact_terms(sentence, &query);
        }
        if let Some(query) = code_shaped_query(block) {
            return inspection_fact_terms(block, &query);
        }
    }
    Vec::new()
}

fn inspection_fact_terms(text: &str, query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let scoped = inspection_subject_and_following(text, query);
    let normalized_query = query.to_lowercase();
    for token in search_tokens(scoped) {
        let normalized = token.replace('-', "_").to_lowercase();
        if normalized.len() < 3
            || normalized == normalized_query
            || normalized.chars().all(|character| character.is_ascii_digit())
            || is_prose_word(&normalized)
            || seed::lexicon().mentions_role(seed::ROLE_WORKSPACE_INSPECTION_ACTION, &normalized)
            || seed::lexicon().mentions_role(seed::ROLE_CODING_SEARCH_SUBJECT_KIND, &normalized)
            || terms.contains(&normalized)
        {
            continue;
        }
        terms.push(normalized);
    }
    if let Some(canonical) = literal_inspection_fact_query(&scoped.to_lowercase())
        && !terms.contains(&canonical)
    {
        terms.push(canonical);
    }
    terms
}

/// The inspection subject and the request that follows it, without a wrapper.
///
/// Harnesses and orchestration layers commonly prefix a task with a numbered or
/// classified label. The code-shaped subject is the first token that belongs to
/// the repository question itself, so starting there removes an arbitrary
/// prefix without having to know any of its words. Hyphenated prose and its
/// underscored source spelling identify the same subject.
fn inspection_subject_and_following<'a>(text: &'a str, query: &str) -> &'a str {
    let hyphenated = query.replace('_', "-");
    [query, hyphenated.as_str()]
        .into_iter()
        .filter_map(|spelling| ascii_case_insensitive_offset(text, spelling))
        .min()
        .and_then(|offset| text.get(offset..))
        .unwrap_or(text)
}

fn ascii_case_insensitive_offset(text: &str, needle: &str) -> Option<usize> {
    needle.is_ascii().then_some(())?;
    text.char_indices()
        .map(|(offset, _)| offset)
        .find(|offset| {
            text.get(*offset..offset + needle.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(needle))
        })
}

fn module_filename_filter(query: &str) -> Option<String> {
    (query.contains('_')
        && query
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'))
    .then(|| format!("*{query}*"))
}

/// Keep source-condition evidence inside production source when the request
/// does not already name a narrower module. Generated traces and tests often
/// quote the same condition verbatim, but are observations about the source
/// rather than the implementation the caller asked to inspect.
fn inspection_filename_filter(text: &str, query: &str) -> Option<String> {
    module_filename_filter(query).or_else(|| {
        let lexicon = seed::lexicon();
        (lexicon.mentions_role(seed::ROLE_CODING_CONDITION_SUBJECT_KIND, text)
            || lexicon.mentions_role(seed::ROLE_CODING_SOURCE_IMPLEMENTATION_SUBJECT_KIND, text))
        .then(|| "src/**/*".to_owned())
    })
}

/// Resolve the source subject named by an inspection request.
///
/// Explicit literal cues and visibly code-shaped tokens are unambiguous on
/// their own. A plain identifier is also unambiguous when the seed lexicon
/// places it next to a source-artifact kind, such as `retry check`.
fn code_shaped_query(text: &str) -> Option<String> {
    let normalized = text.to_lowercase();
    literal_code_search_query(&normalized)
        .or_else(|| shaped_code_search_token(text))
        .or_else(|| adjacent_code_search_token(text, &normalized))
}

/// The longest search cue `normalized` spells out, if it spells one out at all.
///
/// Both registries describe the same act from different sides — the shell
/// vocabulary lowers it to `rg`, the capability registry routes it to a `grep`
/// tool — so a cue from either one is an explicit request to search. The longest
/// match wins because a longer cue is the more specific reading of the same
/// words.
fn longest_search_cue(normalized: &str, vocab: &ShellIntentVocabulary) -> Option<String> {
    let intent_cues = vocab
        .intents
        .iter()
        .filter(|intent| intent.command == "rg")
        .flat_map(|intent| intent.cues.iter());
    let capability_registry = seed::agentic_tool_capabilities();
    let capability_cues = capability_registry
        .iter()
        .filter(|capability| capability.id == "grep")
        .flat_map(|capability| capability.cues.iter());
    intent_cues
        .chain(capability_cues)
        .filter(|cue| normalized.contains(cue.as_str()))
        .max_by_key(|cue| cue.chars().count())
        .cloned()
}

fn literal_code_search_query(normalized: &str) -> Option<String> {
    seed::lexicon()
        .role_word_forms(seed::ROLE_CODING_SEARCH_LITERAL_QUERY)
        .into_iter()
        .find(|form| normalized.contains(&form.text.to_lowercase()))
        .and_then(|form| (!form.action.is_empty()).then(|| form.action.clone()))
}

fn literal_inspection_fact_query(normalized: &str) -> Option<String> {
    seed::lexicon()
        .role_word_forms(seed::ROLE_CODING_SEARCH_FACT_QUERY)
        .into_iter()
        .find(|form| normalized.contains(&form.text.to_lowercase()))
        .and_then(|form| (!form.action.is_empty()).then(|| form.action.clone()))
}

/// Recover the quoted field key from a relationship-serialization question.
///
/// In `how parent relationships are encoded`, `parent` is not merely a broad
/// prose term: it is the literal key whose representation the caller wants to
/// inspect. Searching for the quoted key reflects source serialization syntax
/// and prevents generic words around it from exhausting a client's result cap.
fn serialized_relationship_fact_query(text: &str) -> Option<String> {
    serialized_relationship_term(text).map(|term| format!(r#""{term}""#))
}

/// The relationship noun whose serialization the request asks to inspect.
///
/// Keep this structural extraction shared with evidence ranking: generic fact
/// terms deliberately discard some grammar, while the relationship immediately
/// before a seed-declared relationship kind is the identity-bearing subject.
pub(super) fn serialized_relationship_term(text: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(text);
    if !lexicon.mentions_role(seed::ROLE_CODING_SERIALIZATION_ACTION, &normalized) {
        return None;
    }
    let tokens = search_tokens(text).collect::<Vec<_>>();
    tokens
        .windows(2)
        .find(|pair| {
            valid_search_identifier(pair[0])
                && lexicon.mentions_role(seed::ROLE_CODING_RELATIONSHIP_SUBJECT_KIND, pair[1])
        })
        .map(|pair| pair[0].to_ascii_lowercase())
}

/// The most identifier-shaped token in `prompt`, if it holds one.
///
/// Prose names an identifier by writing it: `task_decomposition`,
/// `retry-policy`, `AgenticPlan`, `Cargo.toml`. Each of those carries a mark
/// that ordinary English words do not, and the marks are weighted by how rarely
/// prose produces them. A hyphen is the weakest because prose hyphenates freely,
/// so `retry-policy` only wins when nothing better is present -- and when it
/// does win it is lowered to `retry_policy`, the spelling the source would use.
///
/// A run of digits separated by dots is excluded even though it carries the
/// strongest mark. It is never an identifier: it is a version, a section, or --
/// the case that sent an entire ladder node to grep for `1.1.1.1.1` and record
/// the result as its evidence -- a node path in the instructions wrapped around
/// the task (issue #1066).
fn shaped_code_search_token(prompt: &str) -> Option<String> {
    search_tokens(prompt)
        .filter_map(|token| {
            let interior_uppercase = token
                .chars()
                .skip(1)
                .any(|character| character.is_ascii_uppercase());
            let score = usize::from(token.contains('.')) * 4
                + usize::from(token.contains('_')) * 4
                + usize::from(interior_uppercase) * 3
                + usize::from(token.contains('-')) * 2;
            let shaped = score > 0 && !token.contains('/') && !is_dotted_number(token);
            shaped.then_some((score, token.len(), token))
        })
        .max_by_key(|(score, length, _)| (*score, *length))
        .map(|(_, _, token)| token.replace('-', "_"))
}

fn adjacent_code_search_token(prompt: &str, normalized: &str) -> Option<String> {
    let tokens = search_tokens_with_offsets(prompt).collect::<Vec<_>>();
    seed::lexicon()
        .role_word_forms(seed::ROLE_CODING_SEARCH_SUBJECT_KIND)
        .into_iter()
        .filter_map(|form| {
            let surface = form.text.to_lowercase();
            let start = normalized.find(&surface)?;
            let end = start.checked_add(surface.len())?;
            tokens
                .iter()
                .filter(|(_, token_start, token_end)| *token_end <= start || *token_start >= end)
                .filter_map(|(token, token_start, token_end)| {
                    let distance = if *token_end <= start {
                        start - *token_end
                    } else {
                        *token_start - end
                    };
                    valid_search_identifier(token).then_some((distance, *token_start, *token))
                })
                .min_by_key(|(distance, offset, _)| (*distance, *offset))
        })
        .min_by_key(|(distance, offset, _)| (*distance, *offset))
        .map(|(_, _, token)| token.to_owned())
}

fn search_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| {
        !character.is_ascii_alphanumeric() && !matches!(character, '_' | '.' | ':' | '-')
    })
    .map(|token| token.trim_matches(['.', '-']))
    .filter(|token| !token.is_empty())
}

fn search_tokens_with_offsets(text: &str) -> impl Iterator<Item = (&str, usize, usize)> {
    text.split_inclusive(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .scan(0, |offset, chunk| {
            let start = *offset;
            *offset += chunk.len();
            let token = chunk.trim_end_matches(|character: char| {
                !character.is_ascii_alphanumeric() && character != '_'
            });
            Some((token, start, start + token.len()))
        })
        .filter(|(token, _, _)| !token.is_empty())
}

fn valid_search_identifier(token: &str) -> bool {
    let mut characters = token.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        && !seed::lexicon()
            .words_for_role(seed::ROLE_CODING_SEARCH_SUBJECT_KIND)
            .iter()
            .any(|surface| surface.eq_ignore_ascii_case(token))
}

/// Resolve a semantic *intent* to its concrete command, backed by the seed
/// [`ShellIntentVocabulary`] (issue #680).
///
/// The two strategies above only fire when the prompt *names* the command (`run
/// pwd`) or asks, in prose, to list a directory. This third strategy handles the
/// common case where the user expresses an intent without naming the tool at all —
/// *"Print the current working directory"* → `pwd`, *"How much disk space is
/// free?"* → `df -h`, *"What is my username?"* → `whoami`. Each intent carries
/// multilingual cue phrases; the first intent whose cue is present in the prompt
/// wins (declaration order is most-specific-first), and its argument — if any — is
/// recovered from the prompt. An intent whose cue matches but whose required
/// argument is absent is skipped so the search continues rather than emitting an
/// argument-less command that would hang (`wc -l` on stdin).
/// The prompt split into sentences, each paired with whether it is a question.
///
/// Sentence boundaries are what separate the user's request from a sentence
/// merely *placed next to* it, so intent cues are read one sentence at a time
/// (issue #907).
fn sentences_with_mood(lower: &str) -> Vec<(&str, bool)> {
    let mut sentences = Vec::new();
    let mut start = 0;
    for (index, character) in lower.char_indices() {
        if !matches!(
            character,
            '.' | '!' | '?' | ';' | '\n' | '。' | '！' | '？' | '；'
        ) {
            continue;
        }
        // A dot inside a token ends nothing — `main.py` is one word, not two
        // sentences.
        if character == '.'
            && lower[index + character.len_utf8()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
        {
            continue;
        }
        let text = lower[start..index].trim();
        if !text.is_empty() {
            sentences.push((text, matches!(character, '?' | '？')));
        }
        start = index + character.len_utf8();
    }
    let tail = lower[start..].trim();
    if !tail.is_empty() {
        sentences.push((tail, false));
    }
    sentences
}

/// The sentences of `lower` that ask or command, dropping those that merely
/// state a fact about one of `cues`.
///
/// The gemini CLI's *"Today's date is Sunday, August 2, 2026 …"* describes the
/// world; describing the date is not asking for it, so no cue inside such a
/// sentence may route (issue #907). Classification is per sentence rather than
/// per cue: once *"the current time is 20:00"* is a statement about *current
/// time*, the shorter cue *time* riding inside it must not route either.
fn requesting_sentences<'a>(
    sentences: &[(&'a str, bool)],
    cues: &[&str],
    vocab: &seed::CallerContextVocabulary,
) -> Vec<&'a str> {
    sentences
        .iter()
        .filter(|(sentence, interrogative)| {
            !cues.iter().any(|cue| {
                sentence.contains(cue) && is_fact_statement(sentence, *interrogative, cue, vocab)
            })
        })
        .map(|(sentence, _)| *sentence)
        .collect()
}

/// Whether `sentence` asks for an artifact to be authored — an authoring verb
/// paired with a program, script or code artifact, as the lexicon evidences
/// those roles in every supported language.
///
/// Issue #907, requirement 3: a turn that carries a task gets the task. A
/// built-in intent riding alongside it ("what is today's date? create a file
/// main.py that prints Hello, world!") answers the smaller of the two questions
/// and silently drops the work, so the intent steps aside for the task.
///
/// [`super::evidence_record`] reads it for the same reason from the other side:
/// a sentence that asks for an artifact to be authored states the work, so it
/// is not a place to deliver some *other* sentence's finding (issue #1066).
pub(super) fn carries_authoring_task(sentence: &str) -> bool {
    use crate::seed::{ROLE_HELLO_WORLD_REFERENCE, ROLE_PROGRAM_KIND, ROLE_PROGRAM_REQUEST};
    let lexicon = seed::lexicon();
    lexicon.mentions_role(ROLE_PROGRAM_REQUEST, sentence)
        && [ROLE_PROGRAM_KIND, ROLE_HELLO_WORLD_REFERENCE]
            .iter()
            .any(|role| lexicon.mentions_role(role, sentence))
}

/// `sentence` without the determiner its subject may open with, so *"the current
/// date is …"* is recognised as a statement about *"current date"*.
fn strip_subject_lead<'a>(sentence: &'a str, vocab: &seed::CallerContextVocabulary) -> &'a str {
    vocab
        .subject_leads
        .iter()
        .find_map(|lead| {
            sentence
                .strip_prefix(lead.as_str())
                .filter(|rest| rest.starts_with(' '))
                .map(str::trim_start)
        })
        .unwrap_or(sentence)
}

/// Whether `sentence` states a fact *about* `cue` instead of requesting it.
///
/// The shape is `<cue> <copula> <value>` (English, Russian, Spanish, Chinese) or
/// `<cue> <value> <copula>` (Hindi): the cue opens the sentence as its subject,
/// a seed-declared copula links it, and a value follows. A question is never a
/// statement — neither one punctuated with `?` nor one that merely carries a
/// seed-declared question word ("我的用户名**是什么**") — and a cue that does not
/// open the sentence is not its subject, so
/// *"tell me what the current time is"* and *"what is the date?"* keep routing.
/// Whether `sentence` *declares where something lives*: the cue is followed by a
/// colon and a single absolute path.
///
/// A copula is not the only way to state a fact — agent harnesses prefer the
/// label form, and Hive Mind's *"Your prepared working directory: /tmp/example"*
/// supplies the working directory exactly as a copula would, which made every
/// production run answer with `pwd` (issue #907, follow-up).
///
/// The value is what separates this from a request, and it has to be: earlier
/// attempts keyed on the colon (which swallowed *"delete the file: old.log"*),
/// on request verbs (a four-word list that rescued nothing), and on subject
/// position (which cannot see that *"count"* and *"search"* are verbs). A
/// harness declares an **absolute** path — that is the only kind worth stating —
/// while a request's argument after a colon is a relative path, a search term,
/// or prose. Every row of the table above stays a request under this rule.
fn labels_a_value(sentence: &str, cue: &str) -> bool {
    let Some(head_end) = sentence.to_lowercase().find(cue) else {
        return false;
    };
    let Some(value) = sentence[head_end + cue.len()..]
        .trim_start()
        .strip_prefix([':', '：'])
        .map(str::trim)
    else {
        return false;
    };
    let mut words = value.split_whitespace();
    let Some(path) = words.next() else {
        return false;
    };
    words.next().is_none() && (path.starts_with('/') || path.starts_with("~/"))
}

fn is_fact_statement(
    sentence: &str,
    interrogative: bool,
    cue: &str,
    vocab: &seed::CallerContextVocabulary,
) -> bool {
    if interrogative || vocab.asks_a_question(sentence) {
        return false;
    }
    if labels_a_value(sentence, cue) {
        return true;
    }
    // The cue may or may not carry the determiner itself ("the current time" vs
    // "current date"), so the subject is tried with and without its lead.
    let Some(rest) = sentence
        .strip_prefix(cue)
        .or_else(|| strip_subject_lead(sentence, vocab).strip_prefix(cue))
    else {
        return false;
    };
    let rest = rest.trim_matches(|character: char| {
        character.is_whitespace() || matches!(character, ',' | ':' | '：' | '，' | '-')
    });
    let words: Vec<&str> = rest.split_whitespace().collect();
    let (Some(head), Some(last)) = (words.first(), words.last()) else {
        return false;
    };
    if let Some(copula) = vocab.copula_in(head) {
        return words.len() > 1 || head.chars().count() > copula.chars().count();
    }
    words.len() > 1 && vocab.copula_in(last).is_some()
}

fn intent_shell_command(prompt: &str, vocab: &ShellIntentVocabulary) -> Option<String> {
    let lower = prompt.to_lowercase();
    let sentences = sentences_with_mood(&lower);
    let caller_context = seed::caller_context_vocabulary();
    let cues: Vec<&str> = vocab
        .intents
        .iter()
        .flat_map(|intent| intent.cues.iter().map(String::as_str))
        .collect();
    let requesting = requesting_sentences(&sentences, &cues, &caller_context);
    // A task the turn carries outranks any built-in intent riding alongside it
    // (issue #907): answering the intent would silently drop the work.
    if carries_authoring_task(&lower) {
        return None;
    }
    // Prefer the most specific matching cue across every intent. This prevents
    // a shorter generic cue (for example "current directory" → `pwd`) from
    // stealing a longer request ("list current directory" → `ls`).
    let (intent, cue) = vocab
        .intents
        .iter()
        .filter(|intent| intent.command != REPORT_ISSUE_ACTION)
        .flat_map(|intent| intent.cues.iter().map(move |cue| (intent, cue)))
        .filter(|(intent, cue)| {
            requesting
                .iter()
                .any(|sentence| sentence.contains(cue.as_str()))
                && (intent.argument != ShellIntentArgument::SearchQuery
                    || vocab
                        .local_search_scopes
                        .iter()
                        .any(|scope| lower.contains(scope)))
        })
        .max_by_key(|(_, cue)| cue.chars().count())?;
    match intent.argument {
        ShellIntentArgument::None => resolve_shell_command(&intent.command, vocab),
        ShellIntentArgument::Path => {
            path_argument(prompt).map(|arg| format!("{} {arg}", intent.command))
        }
        ShellIntentArgument::NameLead => name_lead_argument(prompt, &vocab.name_leads)
            .map(|arg| format!("{} {arg}", intent.command)),
        ShellIntentArgument::OnePath => {
            path_arguments(prompt, cue, vocab, 1).map(|args| format!("{} {args}", intent.command))
        }
        ShellIntentArgument::TwoPaths => {
            path_arguments(prompt, cue, vocab, 2).map(|args| format!("{} {args}", intent.command))
        }
        ShellIntentArgument::Remainder => remainder_argument(prompt, &lower, cue)
            .map(|arg| format!("{} --fixed-strings -- '{arg}' .", intent.command)),
        ShellIntentArgument::SearchQuery => local_search_query(prompt, cue, vocab)
            .map(|arg| format!("{} --fixed-strings -- '{arg}' .", intent.command)),
    }
}

fn path_arguments(
    prompt: &str,
    cue: &str,
    vocab: &ShellIntentVocabulary,
    count: usize,
) -> Option<String> {
    let lower = prompt.to_lowercase();
    let anchored = names_a_path_object(cue, vocab);
    let after_cue = lower
        .find(cue)
        .and_then(|start| prompt.get(start + cue.len()..))
        .unwrap_or_default();
    let mut arguments = collect_path_arguments(after_cue, "", vocab, count, anchored);
    if arguments.len() < count {
        arguments = collect_path_arguments(prompt, cue, vocab, count, anchored);
    }
    (arguments.len() == count).then(|| arguments.join(" "))
}

/// Whether the matched cue names the filesystem object its operands refer to.
///
/// *"Copy the file a.txt to b.txt"* says outright that what follows is a file, so
/// any word may be the name. *"Copy"* on its own says nothing, and issue #863
/// showed what that costs: *"how to do copy stdin to stdout in Rust"* routed to
/// `cp stdin stdout`, and issue #862 turned a Rosetta Code URL into
/// `cp _stdin_to_stdout Rust`. An unanchored cue therefore only accepts operands
/// that [look like paths](looks_like_a_path) — which is also what makes a bare
/// *move* cue safe enough to add for issue #824.
fn names_a_path_object(cue: &str, vocab: &ShellIntentVocabulary) -> bool {
    vocab
        .path_objects
        .iter()
        .any(|object| cue.contains(object.as_str()))
}

/// Whether a token is written the way a path is written: rooted at the home
/// directory, carrying a separator, or ending in an extension. A plain word
/// (`stdin`, `Rust`) is not.
fn looks_like_a_path(token: &str) -> bool {
    if is_dotted_number(token) {
        return false;
    }
    token.starts_with('~')
        || token.contains('/')
        || token.rsplit_once('.').is_some_and(|(stem, extension)| {
            !stem.is_empty()
                && !extension.is_empty()
                && extension.chars().all(char::is_alphanumeric)
        })
}

fn collect_path_arguments(
    text: &str,
    cue: &str,
    vocab: &ShellIntentVocabulary,
    count: usize,
    anchored: bool,
) -> Vec<String> {
    let cue = cue.to_lowercase();
    let mut arguments = Vec::new();
    for word in text.split_whitespace() {
        let candidate = trim_trailing_sentence_dot(
            word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ';' | ':' | '!' | '?')),
        );
        let normalized = candidate.to_lowercase();
        if candidate.is_empty()
            || !is_safe_path(candidate)
            || (!anchored && !looks_like_a_path(candidate))
            || cue.split_whitespace().any(|part| part == normalized)
            || vocab
                .argument_noise
                .iter()
                .any(|noise| noise == &normalized)
        {
            continue;
        }
        arguments.push(candidate.to_owned());
        if arguments.len() == count {
            break;
        }
    }
    arguments
}

fn local_search_query(prompt: &str, cue: &str, vocab: &ShellIntentVocabulary) -> Option<String> {
    let cue_words: Vec<String> = cue.split_whitespace().map(str::to_lowercase).collect();
    let scope_words: Vec<String> = vocab
        .local_search_scopes
        .iter()
        .flat_map(|scope| scope.split_whitespace().map(str::to_lowercase))
        .collect();
    let query = prompt
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric() && !matches!(c, '_' | '-' | '.' | ':'))
        })
        .filter(|word| !word.is_empty())
        .filter(|word| {
            let lower = word.to_lowercase();
            !cue_words.contains(&lower)
                && !scope_words.contains(&lower)
                && !vocab.argument_noise.contains(&lower)
        })
        .collect::<Vec<_>>()
        .join(" ");
    (!query.is_empty()
        && query.chars().all(|c| {
            c.is_alphanumeric() || c.is_whitespace() || matches!(c, '_' | '-' | '.' | ':')
        }))
    .then_some(query)
}

fn resolve_shell_command(command: &str, vocab: &ShellIntentVocabulary) -> Option<String> {
    let action = command.strip_prefix("formal-ai:workspace-");
    if let Some(action) = action {
        return workspace_command(action, vocab).map(str::to_owned);
    }
    Some(command.to_owned())
}

fn workspace_command<'a>(action: &str, vocab: &'a ShellIntentVocabulary) -> Option<&'a str> {
    vocab
        .workspace_commands
        .iter()
        .find(|commands| std::path::Path::new(&commands.marker).is_file())
        .and_then(|commands| match action {
            "test" => Some(commands.test.as_str()),
            "install" => Some(commands.install.as_str()),
            "build" => Some(commands.build.as_str()),
            _ => None,
        })
}

/// Recover a safe literal query following a matched semantic cue.
fn remainder_argument(prompt: &str, lower: &str, cue: &str) -> Option<String> {
    let start = lower.find(cue)? + cue.len();
    let remainder = prompt.get(start..)?.trim();
    (!remainder.is_empty()
        && remainder.chars().all(|c| {
            c.is_alphanumeric() || c.is_whitespace() || matches!(c, '_' | '-' | '.' | ':')
        }))
    .then(|| remainder.to_owned())
}

/// The first filename-looking token in the prompt: a token carrying an interior dot
/// (`Cargo.toml`, `src/lib.rs`) that is a safe relative path, not a URL or flag.
/// Used to fill a [`ShellIntentArgument::Path`] argument (`wc -l Cargo.toml`).
fn path_argument(prompt: &str) -> Option<String> {
    prompt
        .split(|c: char| c.is_whitespace())
        .map(|word| word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ';' | '?')))
        .map(|word| word.trim_end_matches(['.', '!']))
        .find(|token| {
            let interior_dot = token.trim_matches('.').contains('.');
            interior_dot && !token.contains("://") && is_safe_path(token)
        })
        .map(str::to_owned)
}

/// The name introduced by a name-lead cue (*called*/*named*/…): the token following
/// the first name-lead word. Used to fill a [`ShellIntentArgument::NameLead`]
/// argument (`mkdir build` from *"create a directory called build"*).
fn name_lead_argument(prompt: &str, name_leads: &[String]) -> Option<String> {
    let words: Vec<&str> = prompt
        .split(|c: char| c.is_whitespace())
        .filter(|w| !w.is_empty())
        .collect();
    let lead_index = words.iter().position(|word| {
        let normalized = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        name_leads.iter().any(|lead| lead == &normalized)
    })?;
    let name = words
        .get(lead_index + 1)?
        .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ';' | '.' | '!' | '?' | ':'));
    (!name.is_empty() && is_safe_path(name)).then(|| name.to_owned())
}

/// Whether a token is a path the request itself supplies: no `..` escape, no
/// leading dash, only path-safe characters.
///
/// Absolute (`/Users/me/Archive`) and home-relative (`~/Code`) paths count.
/// Issue #824 reported *"Move /Users/…/hive-control-center to ~/Code/…"* refused,
/// and the reason was here: both operands were rejected before any policy got to
/// see them. Every caller recovers the token verbatim from the user's own words,
/// so excluding absolute paths never stopped Formal AI from reaching outside the
/// workspace — it only stopped the user from saying where. `..` stays excluded,
/// because a traversal is a way of *not* saying where.
fn is_safe_path(token: &str) -> bool {
    let body = token.strip_prefix("~/").unwrap_or(token);
    let body = body.strip_prefix('/').unwrap_or(body);
    !token.starts_with('-')
        && !body.is_empty()
        && !body.split('/').any(|part| part == ".." || part.is_empty())
        && token
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | '~'))
}

/// Extract an explicit shell command named in the prompt, backed by the seed
/// [`TerminalCommandVocabulary`].
///
/// A prompt names a command when it carries *run context* — a run/execute verb, a
/// Chinese run verb, or a terminal/shell phrase — together with a recognized shell
/// token (`pwd`, `git`, `cargo`, …). Two shapes are handled:
///
/// * **Verb-adjacent** (`execute pwd`, `run git status`): the shell token immediately
///   follows a run verb, so the token *and its trailing arguments* are the command —
///   arguments run until a natural-language word ([`is_prose_word`]) ends them, so
///   `run git status` → `git status` but `run ls then stop` → `ls`.
/// * **Mentioned** (`Run the ls command to list files`): the token appears with run
///   context but is not directly after the verb, so only the single token is emitted
///   (`ls`) — the surrounding words are prose describing the request, not arguments.
///
/// Both shapes are read **one sentence at a time** (issue #907, follow-up). Run
/// context is only context for the command that shares its sentence: a caller
/// policy clause — *"When running sudo commands, run them in the background."* —
/// pairs a run verb with the `sudo` token across the whole message, and matching
/// them message-wide planned a bare `sudo` for every Codex run through Hive Mind.
/// A sentence that only *governs* commands is also not a request for one, so a
/// conditional clause never licenses the token it mentions.
fn named_shell_command(prompt: &str, vocab: &TerminalCommandVocabulary) -> Option<String> {
    sentence_spans(prompt)
        .into_iter()
        .filter(|sentence| !states_a_command_policy(sentence))
        .find_map(|sentence| named_shell_command_in_sentence(sentence, vocab))
}
