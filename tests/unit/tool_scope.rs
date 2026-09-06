//! Where an advertised tool's effect lands (issue #1075).
//!
//! Deliberately held out from the failing sessions: none of the names,
//! repositories or paths below appear in them. What is under test is the model
//! -- required identities first, address second -- not a memory of three URLs.

use formal_ai::seed::parse_tool_resource_scopes;
use formal_ai::tool_scope::{
    ToolResourceScope, grounded_identity_argument, is_identity_argument,
    scope_of_advertised_tool, scope_of_tool_definition, scope_of_tool_name,
    ungrounded_identity_arguments,
};
use serde_json::{Map, Value, json};

#[test]
fn a_required_repository_makes_a_tool_remote_however_it_is_named() {
    // The name says nothing: `store_document` could be a local writer.
    let definition = json!({
        "name": "store_document",
        "parameters": {
            "type": "object",
            "required": ["project_id", "body"],
            "properties": {"project_id": {"type": "string"}, "body": {"type": "string"}}
        }
    });

    assert_eq!(
        scope_of_tool_definition(&definition, "store_document"),
        ToolResourceScope::RemoteService
    );
}

#[test]
fn a_required_session_makes_a_tool_process_input() {
    let definition = json!({
        "name": "send_text",
        "input_schema": {
            "type": "object",
            "required": ["shell_id", "chars"],
            "properties": {"shell_id": {"type": "number"}, "chars": {"type": "string"}}
        }
    });

    assert_eq!(
        scope_of_tool_definition(&definition, "send_text"),
        ToolResourceScope::ProcessInput
    );
}

#[test]
fn a_plain_file_writer_stays_in_the_workspace() {
    let definition = json!({
        "type": "function",
        "function": {
            "name": "save_file",
            "parameters": {
                "type": "object",
                "required": ["path", "content"],
                "properties": {"path": {"type": "string"}, "content": {"type": "string"}}
            }
        }
    });

    assert_eq!(
        scope_of_tool_definition(&definition, "save_file"),
        ToolResourceScope::ClientWorkspace
    );
    assert!(scope_of_tool_definition(&definition, "save_file").is_client_workspace());
}

#[test]
fn only_whole_address_segments_route_a_call_off_the_workspace() {
    // Addressed *through* a connector namespace.
    assert_eq!(
        scope_of_tool_name("mcp__team_apps__gitlab.create_snippet"),
        ToolResourceScope::RemoteService
    );
    assert_eq!(
        scope_of_tool_name("notion.append_block"),
        ToolResourceScope::RemoteService
    );
    // A local tool that merely mentions a service in one segment is local: it
    // reads a file that happens to be named after one.
    assert_eq!(
        scope_of_tool_name("read_gitlab_config"),
        ToolResourceScope::ClientWorkspace
    );
    assert_eq!(
        scope_of_tool_name("write_notion_export"),
        ToolResourceScope::ClientWorkspace
    );
}

#[test]
fn a_definition_that_was_never_advertised_falls_back_to_its_address() {
    let advertised = [json!({"name": "edit_file", "parameters": {"type": "object"}})];
    assert_eq!(
        scope_of_advertised_tool(&advertised, "slack.post_message"),
        ToolResourceScope::RemoteService
    );
    assert_eq!(
        scope_of_advertised_tool(&advertised, "edit_file"),
        ToolResourceScope::ClientWorkspace
    );
}

#[test]
fn an_identity_names_a_resource_and_a_description_does_not() {
    for identity in ["repository_full_name", "channel_id", "PID", "Issue_Number"] {
        assert!(is_identity_argument(identity), "{identity}");
    }
    for description in ["content", "message", "body", "title", "query"] {
        assert!(!is_identity_argument(description), "{description}");
    }
}

#[test]
fn an_identity_the_request_states_is_grounded_from_it() {
    let request = "Please open a merge request against https://gitlab.com/orbit/tooling/-/issues/42.";
    assert_eq!(
        grounded_identity_argument("repository_full_name", request),
        Some(Value::String(String::from("orbit/tooling")))
    );
    assert_eq!(
        grounded_identity_argument("owner", request),
        Some(Value::String(String::from("orbit")))
    );
    assert_eq!(
        grounded_identity_argument("repo", request),
        Some(Value::String(String::from("tooling")))
    );
}

#[test]
fn an_identity_the_request_never_states_stays_ungrounded() {
    // No URL, so nothing names a repository. The honest answer is "unknown",
    // not the empty string GitHub answered 404 to.
    assert_eq!(
        grounded_identity_argument("repository_full_name", "Tidy up the changelog."),
        None
    );
    // A URL, but it names no channel, so the Slack identity stays unknown too.
    assert_eq!(
        grounded_identity_argument("channel_id", "See https://github.com/orbit/tooling."),
        None
    );
}

#[test]
fn an_ungrounded_identity_is_reported_as_a_missing_precondition() {
    let definition = json!({
        "name": "post_update",
        "parameters": {
            "type": "object",
            "required": ["channel_id", "text"],
            "properties": {"channel_id": {"type": "string"}, "text": {"type": "string"}}
        }
    });

    let missing = ungrounded_identity_arguments(&definition, &Map::new(), "Announce the release.");
    assert_eq!(missing, vec![String::from("channel_id")]);

    // Supplied by the caller, so the precondition holds.
    let mut provided = Map::new();
    provided.insert(String::from("channel_id"), json!("C0ABCDEF"));
    assert!(ungrounded_identity_arguments(&definition, &provided, "Announce the release.").is_empty());

    // Supplied as the empty string, which is the defect itself: present in the
    // JSON, absent as an address.
    let mut blank = Map::new();
    blank.insert(String::from("channel_id"), json!(""));
    assert_eq!(
        ungrounded_identity_arguments(&definition, &blank, "Announce the release."),
        vec![String::from("channel_id")]
    );
}

#[test]
fn the_vocabulary_is_read_from_seed_data_rather_than_compiled_in() {
    // A deployment that adds a connector adds a line to the seed file; nothing
    // in `src/` has to learn the name. Parsing a small document proves the
    // vocabulary is the thing being consulted.
    let vocabulary = parse_tool_resource_scopes(
        "tool_resource_scopes\n  scope remote_service\n    arguments (\"board_id\")\n    namespaces (\"kanban\")\n  scope process_input\n    arguments (\"repl_id\")\n    names (\"feed_repl\")\n  identity\n    arguments (\"board_id\" \"repl_id\")\n",
    );

    assert_eq!(vocabulary.remote_arguments, vec![String::from("board_id")]);
    assert_eq!(vocabulary.remote_namespaces, vec![String::from("kanban")]);
    assert_eq!(vocabulary.process_names, vec![String::from("feed_repl")]);
    assert_eq!(
        vocabulary.identity_arguments,
        vec![String::from("board_id"), String::from("repl_id")]
    );
}

#[test]
fn a_bare_address_grounds_the_argument_that_names_it() {
    // A URL is an identity too, and a request that carries one has stated it.
    // Withholding a fetch tool because its required `url` "could not be
    // grounded" would ground nothing and lose the capability -- the grounding
    // rule is about what the request does not say, not about what it does.
    assert_eq!(
        grounded_identity_argument("url", "retrieve https://docs.example.org/status."),
        Some(json!("https://docs.example.org/status"))
    );
    assert_eq!(
        grounded_identity_argument("uri", "(see https://packages.example.net/index)"),
        Some(json!("https://packages.example.net/index"))
    );
    assert_eq!(
        grounded_identity_argument("url", "summarise the release notes"),
        None
    );
}
