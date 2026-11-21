//! Capability-first routing for the shared agentic CLI tool set (issue #758).

use serde_json::json;

use super::general_planner::compose_edit_request;
use super::planner::{AgenticPlan, Capability, plan_one};
use crate::capability_routing::RoutingOutcome;
use crate::protocol::ChatMessage;
use crate::seed;

#[path = "capability_router/tool_classification.rs"]
mod tool_classification;

pub(super) use tool_classification::{
    classify_tool, is_workspace_creation_tool, shell_command_tool, tool_for,
    workspace_creation_tool,
};

/// Protocol-native research tools a provider executes on its own servers.
const HOSTED_RESEARCH_TOOLS: [&str; 5] = [
    "web_search",
    "web_fetch",
    "web_search_preview",
    "file_search",
    "computer_use_preview",
];

/// Whether `name` is a protocol-native research tool the provider itself runs.
pub(super) fn is_hosted_research_tool(name: &str) -> bool {
    HOSTED_RESEARCH_TOOLS
        .iter()
        .any(|hosted| hosted.eq_ignore_ascii_case(name))
}

/// Where in the planner a routed request is answered.
///
/// The decision table speaks about every request, but not at the same moment
/// about all of them. A request whose effect lands in the *workspace*, or whose
/// object is a URL, a path, a container or a literal, names something the
/// research routers cannot know better -- and letting them claim it on the
/// strength of the sentence's shape is precisely the #745 and #758 defect. Such
/// a request is answered before them. Everything else -- a bare term the open
/// web has to answer -- is the research routers' own subject, and the table
/// answers it only once they have declined, so a multi-turn research recipe is
/// never cut short by a single search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RoutingStage {
    /// Before the research routers: the workspace and the named objects.
    NamedOrLocal,
    /// After them: a bare term whose answer is on the open web.
    OpenWeb,
}

/// Whether the request names a filesystem container and asks nothing of it.
///
/// `acts` appends `retrieve` to every request, because a request that evidences
/// no narrower act is a retrieval -- which is the right default for an object
/// the request *named*, and the wrong one for a container it merely mentioned.
/// "Execute everything in the workspace" names a workspace and asks for no act
/// the seed knows, and answering it with a bare `ls` is passing prose through
/// as a command (issue #907). A retrieval the request *asked for* in the
/// seed's own words -- "busca X en mi escritorio", "… में खोजिए" -- is not
/// that case, however little else the sentence names: the #840 ladder's hi/es
/// folder nodes must reach the same `list_dir` lowering the English node does
/// (plan 10 leaf 21).
fn names_a_container_without_an_act(task: &str) -> bool {
    use crate::capability_routing::{Act, ObjectType, acts, evidences_retrieve_act, object_type};
    object_type(task).first().copied() == Some(ObjectType::PathScope)
        && acts(task) == [Act::Retrieve]
        && !evidences_retrieve_act(task)
}

/// Which stage a request belongs to, read from the derivations rather than from
/// the capability the table happens to name.
fn stage_of(task: &str) -> RoutingStage {
    use crate::capability_routing::{Locus, ObjectType, locus, object_type};
    let highest = object_type(task).first().copied().unwrap_or_default();
    let named = matches!(
        highest,
        ObjectType::Url
            | ObjectType::Path
            | ObjectType::PathScope
            | ObjectType::QuotedContent
            | ObjectType::Pattern
            | ObjectType::PathSet
            | ObjectType::TaskList
            | ObjectType::Delegation
    );
    if named || locus(task) == Locus::Workspace {
        RoutingStage::NamedOrLocal
    } else {
        RoutingStage::OpenWeb
    }
}

/// The capability slugs `data/seed/capability-routing.lino` names, paired with
/// the client capability that answers each. A slug with no pairing here is one
/// the decision table routes to but no advertised tool provides -- the calendar,
/// the response-language demonstration, the measurement lookup -- and the table
/// reports an honest gap for it rather than a tool call.
///
/// Plan 10 leaf 11: the seven request-side cue families joined this table, so
/// every capability the seed registry declares is decided by a
/// `(object, act, locus)` row and none by a memorized phrase.
/// Capabilities that only observe the workspace: a request naming a mutating
/// shell intent is never answered by one of them.
const OBSERVING_SLUGS: [&str; 4] = ["read_many", "list_dir", "glob", "grep"];

const ROUTED_CAPABILITIES: [(&str, Capability); 12] = [
    ("web_fetch", Capability::Fetch),
    ("web_search", Capability::Search),
    ("read_file", Capability::Read),
    ("write_file", Capability::Write),
    ("list_dir", Capability::ListDir),
    ("grep", Capability::Grep),
    ("shell", Capability::Run),
    ("glob", Capability::Glob),
    ("read_many", Capability::ReadMany),
    ("multi_edit", Capability::MultiEdit),
    ("todo", Capability::Todo),
    ("subagent", Capability::Subagent),
];

/// Route one request through the decision table of
/// `data/seed/capability-routing.lino` (issue #1138 B10, plan 10 leaves 9-11).
///
/// Since leaf 11 this is the only request-side capability decision there is:
/// the 280 memorized cue phrases and the `task_matches` phrase scan are gone,
/// and every capability the seed registry declares is chosen by an
/// `(object, act, locus)` row. A triple with no row returns [`None`] and the
/// older routes see the request, which is how a class with no row yet stays
/// visible instead of silently guessed.
///
/// Only the capabilities a tool can actually satisfy are advertised to
/// [`crate::capability_routing::route`], so a row naming a capability this
/// client has no tool for produces an honest gap there rather than a tool call
/// here.
pub(super) fn plan_routed_capability_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    stage: RoutingStage,
) -> Option<AgenticPlan> {
    plan_routed_capability_step_in(task, messages, tool_names, stage, &[])
}

/// The capability slugs the retired cue arm used to decide, by slug.
///
/// Plan 10 leaf 11: the phrases are gone, but these seven rows have to be
/// decided at the arm's old position -- ahead of the shell cascade and the
/// file-read fallback -- because each of their rows names a specialized tool
/// with a `fallback shell`, and a request the table sends to an advertised
/// `grep_search`, `list_directory`, `glob`, `read_many_files`, `multi_edit`,
/// `todo_write` or `task` must not be answered by the shell lowering the same
/// row declares (issue #758's specialized-first policy, now carried by the
/// rows' `fallback` fields and this position instead of a phrase scan).
const NAMED_CAPABILITY_SLUGS: [&str; 7] = [
    "grep",
    "glob",
    "list_dir",
    "read_many",
    "multi_edit",
    "todo",
    "subagent",
];

/// The seven named capabilities, decided by the table at the position the cue
/// arm held: after the routes that read the conversation, before the shell
/// cascade that would otherwise lower the same request to `bash`.
pub(super) fn plan_named_capability_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    plan_routed_capability_step_in(
        task,
        messages,
        tool_names,
        RoutingStage::NamedOrLocal,
        &NAMED_CAPABILITY_SLUGS,
    )
}

fn plan_routed_capability_step_in(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    stage: RoutingStage,
    only: &[&str],
) -> Option<AgenticPlan> {
    // The capability belongs to the stated request, not to a later harness
    // block that only places the worker. Every agentic research route uses the
    // same block boundary; applying the table to the whole envelope let "work
    // only in this checkout" turn a live exchange-rate query into a local
    // directory request (issue #1066).
    let routed_task = super::stated_request::request_blocks(task)
        .into_iter()
        .next()
        .unwrap_or(task);
    let first_block = routed_task;
    // A sentence that merely governs commands is the caller's framing, not
    // the work: the task sentence after it states the request, and a policy
    // sentence's own nouns ("files", "workspace") must not name a capability
    // for the turn (issue #907's policy class -- the same sentence filter the
    // shell route's reader applies). A prompt that is all policy keeps its
    // whole shape, so the boundary below still declines it as governed.
    let policy_free_request = super::shell_command_policy::sentence_spans(routed_task)
        .into_iter()
        .filter(|sentence| !super::shell_command_policy::states_a_command_policy(sentence))
        .collect::<Vec<_>>()
        .join(" ");
    let routed_task: &str = if policy_free_request.is_empty() {
        routed_task
    } else {
        &policy_free_request
    };
    if !crate::capability_routing::table_routing_enabled() || stage_of(routed_task) != stage {
        return None;
    }
    // A prompt that dictates a shell command verbatim (`execute cp a.txt b.txt`)
    // is not a request whose operands the table may re-read: it belongs to the
    // shell cascade's passthrough, so the planner must not lower it through a
    // capability row first (issue #749).
    if super::shell_command::explicit_passthrough_command(routed_task).is_some() {
        return None;
    }
    // At the open-web position every route that reads the conversation and the
    // workspace has declined, and a request that never named the open web is
    // one the symbolic engine should still answer. "What is Links Notation?"
    // is not a search just because nothing above claimed it (issue #989): the
    // seed's concept lookup resolves it, so the engine speaks; a computation
    // ("What is 480 divided by 15?") is held the same way. A definition the
    // seed's own lookup leaves unresolved is the honest unknown that only a
    // trusted external source answers, whatever language asks it -- "Что
    // такое фуфломицин?" searched in the ladder's committed baseline -- and a
    // current-fact request the table itself routed to `(bare_term, retrieve,
    // web)` ("Verify the current exchange rate between the euro and the yen",
    // the news class) is that same admission in the table's own words (plan
    // 10 leaf 21, issue #840). A measurement the table honestly gaps is still
    // refused below by its own outcome, never downgraded to a search.
    let engine_answerable_concept = crate::concepts::extract_concept_query(routed_task).is_some()
        && !super::web_research::concept_lookup_leaves_unknown(routed_task);
    // The concept half above answers definitions; the fact store answers
    // questions its records name -- "What is the capital of France?" is the
    // solver's own `fact_lookup` row, not a search (issue #1138: the gate
    // read only the concept lookup, so a fact the engine owned leaked to the
    // open web as soon as no concept matched).
    let engine_answerable_fact = crate::solver_handlers::fact_store_resolves(routed_task);
    // The catalog half: a request the code catalog itself can template is a
    // write the coding routes own. Deferring code creation to the typed recipe
    // path leaves such a request to the solver, and when the table answered it
    // instead, "Write a hello world program in Python." over a full toolset
    // became a web search for the whole sentence (issue #907) -- the boundary
    // is the catalog, because the recipe path also composes programs no
    // contract pins.
    let engine_answerable_program = super::code_artifact::catalog_claims(routed_task);
    // The dialogue half: "hi" or "thanks" on its own is an exchange the engine
    // answers in the reply, not a bare term for the open web.
    let engine_answerable_dialogue = crate::capability_routing::is_dialogue_utterance(routed_task);
    if stage == RoutingStage::OpenWeb
        && !crate::capability_routing::names_open_web(routed_task)
        && (engine_answerable_concept
            || engine_answerable_fact
            || engine_answerable_program
            || engine_answerable_dialogue)
    {
        return None;
    }
    let advertised: Vec<&str> = ROUTED_CAPABILITIES
        .iter()
        .filter(|(_, capability)| tool_for(tool_names, *capability).is_some())
        .map(|(slug, _)| *slug)
        .collect();
    let (slug, lowered_from) = match crate::capability_routing::route(routed_task, &advertised) {
        RoutingOutcome::Routed { capability } => (capability, None),
        RoutingOutcome::Lowered {
            preferred,
            capability,
        } => (capability, Some(preferred)),
        RoutingOutcome::HonestGap { .. } | RoutingOutcome::Ask { .. } => return None,
    };
    // A positional restriction reads the capability the row *decided on* -- the
    // preferred one when the row lowered, the row's own otherwise. A shell
    // outcome reached without a named preferred capability (a shell row) is
    // never a named-capability decision and stays out of the restricted call.
    let decided = lowered_from.as_deref().unwrap_or(&slug);
    if !only.is_empty() && !only.contains(&decided) {
        return None;
    }
    let capability = ROUTED_CAPABILITIES
        .iter()
        .find(|(name, _)| *name == slug)
        .map(|(_, capability)| *capability)?;
    if names_a_container_without_an_act(routed_task) {
        return None;
    }
    // A batch read is a request to *read*. A request the shell-intent
    // vocabulary recognizes as mutating — a requesting sentence names the
    // cue of an intent that declares an effect, whether or not the operands
    // resolve safely enough to build the command — is not a batch read, and
    // this arm must not answer it, not even from the latest turn's result:
    // a mutating request is carried out as the verified recipe its seed
    // intent declares (issues #824 and #944), and the table may not end a
    // recipe already under way (issue #781). The boundary is the seed's own
    // cue and effect declarations, so every mutating intent in every
    // language the vocabulary covers defers here without being named in
    // Rust, while a genuine batch read ("read all of these files: a.txt and
    // b.md") names no mutating cue and keeps its route (issue #1021:
    // "copy a.txt to b.txt" planned `cat`, and a traversal the safety rule
    // refused answered as a one-file cat of the operand that survived).
    // Every observing capability defers the same way: "Create a directory
    // named src." routed to `list_dir` on the word "directory" and answered
    // with a listing instead of `mkdir src` (PR #1188 dogfooding). A run
    // defers too: "Rename the file a to b" is the shell arm's verified recipe,
    // never a bare `mv` (PR #1188 G24).
    if (capability == Capability::ReadMany
        || capability == Capability::Run
        || OBSERVING_SLUGS.contains(&decided))
        && super::shell_command::names_mutating_shell_intent(routed_task)
    {
        return None;
    }
    // ReadMany requires structural Read operands from the full original request.
    if decided == "read_many" && routed_read_many_paths(task).is_empty() {
        return None;
    }
    if super::tool_result::has_latest_turn_result(messages) {
        return super::tool_result::latest_turn_answer(messages, tool_names, task)
            .map(AgenticPlan::Final);
    }
    // A sentence that *governs* commands is not one that requests one: "Never
    // run chmod on files outside the workspace" names a workspace and a file and
    // asks for neither (issues #907, #916). The shell route already draws this
    // boundary and it is drawn here for the same capabilities, not a second way.
    if capability == Capability::Run
        && super::shell_command_policy::governs_commands_rather_than_requesting_one(routed_task)
    {
        return None;
    }
    // A lowering to the generic listing yields to a more specific shell intent:
    // "What is current directory?" asks *where the session is*, not what the
    // place holds, and the seed's shell-intent vocabulary answers that with
    // `pwd` (issue #989) — a question the container row's `ls` cannot. The
    // #907 rule above keeps containers the request merely mentioned out of
    // the table; this one keeps the session's own place out of it too, and
    // lets every genuine listing ("show me the files in the folder") through,
    // because only an intent the vocabulary resolves *differently* defers.
    if capability == Capability::Run
        && lowered_from.as_deref() == Some("list_dir")
        // The comparison target is the very command the row's lowering runs,
        // so the defer tracks the seed's listing command instead of naming it
        // here (the literal-predicate ratchet, issue #1085 D1).
        && let Some(listing) = shell_fallback(Capability::ListDir, routed_task)
        && super::shell_command::semantic_shell_command_for_task(routed_task)
            .is_some_and(|command| !command.starts_with(&listing))
    {
        return None;
    }
    // A file the request asks to *create* cannot be read: the target does not
    // exist yet, so reading it is always the wrong tool (issue #681). This is
    // the rule `file_read_task_for` already applies, reused rather than
    // re-derived.
    // The rule is about a file the request asks to *create*, so it needs the
    // file as well as the verb. A request that states a write verb and names no
    // destination is asking for something to be shown, not made: Spanish
    // "escribe el contenido de sample.txt" and Chinese "输出内容 sample.txt" both
    // head their read with a verb the seed also knows as a write action, and
    // refusing the read on the verb alone left them with no route at all.
    if capability == Capability::Read
        && super::write_request::states_write_action(routed_task)
        && super::write_request::stated_write_target(routed_task).is_some()
        && (super::general_planner::has_file_write_intent(routed_task)
            || !retrieval_owns_write_cue(routed_task))
    {
        return None;
    }
    // Nor is it one of several files to read (PR #1188 G102).
    if decided == "read_many"
        && super::write_request::states_write_action(routed_task)
        && super::write_request::stated_write_target(routed_task)
            .is_some_and(|target| file_tokens(routed_task).contains(&target.as_str()))
    {
        return None;
    }
    let tool = tool_for(tool_names, capability)?;
    // Classification may omit policy; writable operands retain original statement spans.
    let operand_task = if decided == "read_many" {
        task
    } else if matches!(capability, Capability::Write | Capability::Read) {
        first_block
    } else {
        routed_task
    };
    let arguments = routed_arguments(capability, lowered_from.as_deref(), operand_task)?;
    let plan = plan_one(tool, arguments);
    if decided == "read_many" && super::file_read::read_policy_blocks_plan(task, &plan) {
        return None;
    }
    Some(plan)
}

/// The command a `grep` lowered to the shell runs when the shell vocabulary
/// does not recognise the request as a search.
///
/// The table has already decided this is code navigation -- the object is a
/// bare term and the locus is the workspace -- so the cue that
/// [`super::shell_command::shell_command_for_task`] looks for has nothing left
/// to decide, and a code-shaped token is enough. Without this, a Spanish code
/// search fell through to a web search, which is the #758 defect in the one
/// language the cue vocabulary had not been given (issue #1138 B10).
fn lowered_search_command(preferred: Capability, task: &str) -> Option<String> {
    if preferred != Capability::Grep {
        return None;
    }
    let query = super::shell_command::code_shaped_query(task)
        .or_else(|| super::shell_command::code_search_query_for_task(task))?;
    Some(format!("rg -n {}", shell_quote(&query)))
}

/// The call the routed capability is made with.
///
/// A lowering builds the command for the capability it lowered *from*, so
/// `list_dir` lowered to the shell still runs `ls` and code navigation lowered
/// to the shell still greps -- the shape #758 asked for, now stated by the row's
/// `fallback` field rather than by a Rust cascade.
fn routed_arguments(
    capability: Capability,
    lowered_from: Option<&str>,
    task: &str,
) -> Option<String> {
    if capability == Capability::Run {
        let preferred = lowered_from
            .and_then(|slug| {
                ROUTED_CAPABILITIES
                    .iter()
                    .find(|(name, _)| *name == slug)
                    .map(|(_, capability)| *capability)
            })
            .unwrap_or(Capability::Run);
        let command = shell_fallback(preferred, task)
            .or_else(|| super::shell_command::shell_command_for_task(task))
            .or_else(|| lowered_search_command(preferred, task))?;
        return Some(json!({ "command": command }).to_string());
    }
    match capability {
        Capability::Fetch => {
            let url = crate::capability_routing::first_url(task)?;
            Some(super::planner::fetch_arguments(&url))
        }
        Capability::Search => {
            let query = super::stated_request::request_blocks(task)
                .into_iter()
                .find_map(super::web_research::open_web_query_for_block)
                .unwrap_or_else(|| {
                    crate::solver_handlers::web_search_intent::clean_search_query(task)
                });
            Some(json!({ "query": query }).to_string())
        }
        Capability::Read => {
            let super::file_read::FileReadTask::Direct { path, .. } =
                super::file_read::file_read_task_for(task)?
            else {
                return None;
            };
            Some(json!({"path": path, "filePath": path, "file_path": path}).to_string())
        }
        Capability::Write => {
            // A removal never writes new content over the file (PR #1188 T29:
            // a line deletion was routed here and the file became one quoted
            // line).
            let outside = crate::engine::normalize_prompt(
                &crate::solver_handlers::text_outside_quoted_segments(task),
            );
            if seed::lexicon().mentions_role("coding_text_remove_action", &outside) {
                return None;
            }
            // Neither does an edit the edit composer could not read: unless the
            // request states a whole-file write, its edit word asks for a change
            // inside the file (U17: `In notes.txt replace with ','` wrote `','`).
            if seed::lexicon().mentions_role(seed::ROLE_FILE_EDIT_ACTION_CUE, &outside)
                && !super::literal_write_guard::writes_whole_file(task)
            {
                return None;
            }
            let bound = super::general_planner::compose_general_change_plan(task)?;
            (bound.mode == super::general_planner::GeneralPlanMode::LiteralFile)
                .then(|| super::planner::write_arguments(&bound.target, &bound.content))
        }
        Capability::MultiEdit => {
            // The table routes a set of files transformed in place here; the
            // edit itself still has to be composable. A request whose edit
            // cannot be composed is declined rather than answered with
            // placeholder bytes (the guard `task_matches` used to apply before
            // the capability was chosen at all).
            let (path, old, new) = compose_edit_request(task)?;
            let paths = file_tokens(task);
            Some(
                json!({
                    "path": path,
                    "paths": paths,
                    "edits": [{
                        "old": old,
                        "new": new,
                        "old_string": old,
                        "new_string": new,
                    }],
                })
                .to_string(),
            )
        }
        Capability::Grep => {
            // The table's `(bare_term, retrieve, workspace)` row and the
            // workspace-inspection route answer one request class, and the
            // named-capability position now reaches it first. The inspection
            // subject rule is the stricter, seed-backed admission, so its
            // canonical literal/fact bindings name the query; without this a
            // bare-term retrieval grep fell back to the raw request sentence
            // (issue #1066).
            if let Some(search) =
                super::workspace_inspection::workspace_inspection_search_for_task(task)
            {
                let mut arguments = json!({
                    "query": search.query,
                    "pattern": search.pattern,
                });
                if let Some(include) = search.include {
                    arguments["include"] = include.into();
                }
                Some(arguments.to_string())
            } else {
                Some(arguments_for(capability, task))
            }
        }
        _ => Some(arguments_for(capability, task)),
    }
}

/// The call a capability that needs no per-request derivation is made with.
///
/// Plan 10 leaf 11: this was the cue capabilities' argument builder behind
/// `plan_shared_capability_step`; the routed step now derives the
/// request-shaped arguments itself (`routed_arguments`) and only these
/// schema-shaped defaults remain.
fn arguments_for(capability: Capability, task: &str) -> String {
    match capability {
        Capability::Grep => {
            let query = super::shell_command::code_search_query_for_task(task)
                .unwrap_or_else(|| task.to_owned());
            json!({"query": query, "pattern": query}).to_string()
        }
        Capability::Glob => {
            let pattern = wildcard_token(task).unwrap_or("*");
            json!({"pattern": pattern, "path": "."}).to_string()
        }
        Capability::ListDir => {
            json!({"path": super::directory_listing::listed_directory(task)}).to_string()
        }
        Capability::Todo => json!({
            "todos": [{"content": task, "status": "pending"}],
            "plan": [{"step": task, "status": "pending"}],
        })
        .to_string(),
        Capability::Subagent => json!({
            "description": task,
            "prompt": task,
            "input": task,
            "subagent_type": "general",
        })
        .to_string(),
        Capability::AskUser => String::new(),
        Capability::ReadMany => {
            let paths = routed_read_many_paths(task);
            json!({"paths": paths, "file_paths": paths}).to_string()
        }
        _ => json!({"prompt": task}).to_string(),
    }
}

fn wildcard_token(task: &str) -> Option<&str> {
    task.split_whitespace()
        .map(|token| token.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '`' | '"' | '\'')))
        .find(|token| token.contains('*') || token.contains('?') || token.contains('['))
}

fn file_tokens(task: &str) -> Vec<&str> {
    task.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '`' | '"' | '\'' | '(' | ')'))
        })
        .filter(|token| {
            token.contains('.')
                && !token.starts_with('.')
                && !token.ends_with('.')
                && !token.contains("//")
                && !token.contains(|c: char| FRAGMENT_MARKS.contains(c))
        })
        .collect()
}

/// Marks that a path never holds: a token that still holds one after
/// trimming is a fragment of quoted text or code (`fn(«x»`), never a file to
/// read (PR #1188 G102; mirrors `FRAGMENT_MARKS`).
const FRAGMENT_MARKS: &str = "«»“”‘’()[]{}<>`\"'";

/// Structural Read operands, without filename-token inference.
fn routed_read_many_paths(task: &str) -> Vec<String> {
    match super::file_read::file_read_task_for(task) {
        Some(super::file_read::FileReadTask::DirectMany { paths, .. }) => paths,
        Some(super::file_read::FileReadTask::Direct { path, .. }) => vec![path],
        _ => Vec::new(),
    }
}

fn shell_fallback(capability: Capability, task: &str) -> Option<String> {
    match capability {
        Capability::Grep => super::shell_command::shell_command_for_task(task),
        Capability::Glob => {
            let pattern = wildcard_token(task).unwrap_or("*").replace('\'', "'\\''");
            let mut command = String::from("find");
            command.push_str(" .");
            command.push_str(" -path ");
            command.push('\'');
            command.push_str(&pattern);
            command.push('\'');
            Some(command)
        }
        Capability::ListDir => {
            let directory = super::directory_listing::listed_directory(task);
            Some(if directory == "." {
                String::from("ls")
            } else {
                format!("ls {}", shell_quote(&directory))
            })
        }
        Capability::ReadMany => {
            let paths = routed_read_many_paths(task);
            (!paths.is_empty()).then(|| {
                let paths = paths
                    .into_iter()
                    .map(|path| shell_quote(&path))
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut command = String::from("cat");
                command.push(' ');
                command.push_str(&paths);
                command
            })
        }
        _ => None,
    }
}

use crate::portable_string_contracts::quote_shell_word as shell_quote;

/// A seeded retrieval phrase may own the very token also used as a write cue.
fn retrieval_owns_write_cue(task: &str) -> bool {
    let tokens = super::write_request::tokens(task);
    let Some(start) = super::write_request::first_action_cue_start(&tokens) else {
        return false;
    };
    let action = crate::engine::normalize_prompt(&task[start..]);
    seed::lexicon()
        .words_for_role(seed::ROLE_CAPABILITY_ACT_RETRIEVE)
        .iter()
        .any(|surface| {
            let cue = crate::engine::normalize_prompt(surface);
            !cue.is_empty() && action.starts_with(&cue)
        })
}
