//! Desktop application surface tests.
//!
//! Issue #280 / R17 requires a desktop path similar to `vk-bot-desktop`, but
//! without forking the symbolic solver. The shell should package the existing
//! web chat, talk to the local OpenAI-compatible HTTP API, expose graph/network
//! diagnostics, keep memory import/export on the shared full-bundle format, and
//! make agent/tool-call permissions explicit.

use formal_ai::{
    environment_records, export_memory_full, handle_api_request, import_memory_full, seed_files,
    BundleInfo, MemoryEvent, MemoryStore,
};

const DESKTOP_PACKAGE: &str = include_str!("../../../desktop/package.json");
const DESKTOP_MAIN: &str = include_str!("../../../desktop/main.cjs");
const DESKTOP_PRELOAD: &str = include_str!("../../../desktop/preload.cjs");
const DESKTOP_SMOKE: &str = include_str!("../../../desktop/scripts/smoke.mjs");
const WEB_APP: &str = include_str!("../../../src/web/app.js");

#[test]
fn desktop_package_declares_local_dev_smoke_and_release_commands() {
    let manifest: serde_json::Value = serde_json::from_str(DESKTOP_PACKAGE).unwrap();
    assert_eq!(manifest["name"], "formal-ai-desktop");
    assert_eq!(manifest["main"], "main.cjs");

    let scripts = manifest["scripts"].as_object().expect("scripts object");
    for command in [
        "dev",
        "build",
        "build:linux",
        "build:mac",
        "build:win",
        "smoke",
    ] {
        assert!(
            scripts.contains_key(command),
            "desktop package must document npm run {command}"
        );
    }

    assert!(
        DESKTOP_SMOKE.contains("contextIsolation") && DESKTOP_SMOKE.contains("/v1/graph"),
        "desktop smoke script should verify shell hardening and graph wiring"
    );
}

#[test]
fn desktop_shell_uses_electron_with_hardened_preload_bridge() {
    assert!(DESKTOP_MAIN.contains("BrowserWindow"));
    assert!(DESKTOP_MAIN.contains("contextIsolation: true"));
    assert!(DESKTOP_MAIN.contains("nodeIntegration: false"));
    assert!(DESKTOP_MAIN.contains("formalAiDesktop:getStatus"));
    assert!(DESKTOP_MAIN.contains("/v1/chat/completions"));
    assert!(DESKTOP_MAIN.contains("/v1/graph"));
    assert!(DESKTOP_MAIN.contains("cargo"));
    assert!(DESKTOP_MAIN.contains("formal-ai"));

    assert!(DESKTOP_PRELOAD.contains("contextBridge"));
    assert!(DESKTOP_PRELOAD.contains("FormalAiDesktop"));
    assert!(DESKTOP_PRELOAD.contains("getStatus"));
}

#[test]
fn desktop_runs_in_process_by_default_with_opt_in_server() {
    // R3: the desktop defaults to the in-process reasoning agent.
    // R4: the local OpenAI-compatible server is off by default and only starts
    // when the user opts in via FORMAL_AI_DESKTOP_SERVER. The /download page
    // copy promises exactly this, so the shell must honour it.
    assert!(
        DESKTOP_MAIN.contains("FORMAL_AI_DESKTOP_SERVER"),
        "main.cjs must gate the local server behind FORMAL_AI_DESKTOP_SERVER"
    );
    assert!(
        DESKTOP_MAIN.contains("serverModeRequested()"),
        "main.cjs must only start the server when explicitly requested"
    );
    assert!(
        DESKTOP_MAIN.contains(r#"mode: "in-process""#),
        "main.cjs default desktop status must report in-process mode"
    );

    // The web surface must keep an in-process path that does not depend on the
    // server: it only routes to the API when both apiReady and apiBase are set.
    assert!(
        WEB_APP.contains("currentDesktopStatus.apiReady && currentDesktopStatus.apiBase"),
        "app.js must only call the desktop server when it is ready, else stay in-process"
    );
}

#[test]
fn desktop_web_surface_exposes_status_permissions_and_http_chat_path() {
    for expected in [
        "FormalAiDesktop",
        "desktop-shell-status",
        "desktop-agent-permission",
        "desktop-tool-permission",
        "desktop-permission-panel-sidebar",
        "command-approval",
        "desktopToolGrants",
        "agentOnboardingSeen",
        "desktop-network-link",
        "requestDesktopAnswer",
        "/v1/chat/completions",
    ] {
        assert!(
            WEB_APP.contains(expected),
            "desktop UI should contain `{expected}`"
        );
    }
}

#[test]
fn desktop_environment_is_declared_in_seed_directory() {
    let desktop = environment_records()
        .into_iter()
        .find(|environment| environment.id == "desktop")
        .expect("desktop environment should be declared in seed directory");
    let searchable = [
        desktop.label,
        desktop.runtime,
        desktop.seed_path,
        desktop.memory_store,
        desktop.memory_export_command,
        desktop.bundle_export_command,
        desktop.bundle_import_command,
        desktop.start_command,
        desktop.package_command,
        desktop.tools.join("|"),
    ]
    .join("\n");
    for expected in [
        "Electron desktop shell",
        "formal-ai serve",
        "v1_chat_completions",
        "v1_graph",
        "agent_permission_gate",
        "formal_ai_bundle",
    ] {
        assert!(
            searchable.contains(expected),
            "environment seed should mention `{expected}`"
        );
    }
}

#[test]
fn desktop_chat_path_reuses_openai_http_completion_endpoint() {
    let body = serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{"role": "user", "content": "Hi"}],
        "stream": false
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert_eq!(response.status_code, 200);

    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "chat.completion");
    assert_eq!(json["choices"][0]["message"]["role"], "assistant");
    assert!(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .contains("Hi, how may I help you?"));
}

#[test]
fn desktop_network_view_reuses_graph_endpoint() {
    let response = handle_api_request("GET", "/v1/graph?trace=answer_greeting_hi", "");
    assert_eq!(response.status_code, 200);

    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert!(json["nodes"]
        .as_array()
        .is_some_and(|nodes| !nodes.is_empty()));
    assert!(json["edges"]
        .as_array()
        .is_some_and(|edges| !edges.is_empty()));
}

#[test]
fn desktop_memory_import_export_round_trips_full_bundle() {
    let mut store = MemoryStore::new();
    store.append(MemoryEvent::user("Hi from desktop"));
    store.append(MemoryEvent::assistant("Hi, how may I help you?"));

    let bundle = export_memory_full(&seed_files(), store.events(), &[], &BundleInfo::default());
    assert!(bundle.starts_with("formal_ai_bundle"));
    let imported = import_memory_full(&bundle);

    assert_eq!(imported.events.len(), 2);
    assert_eq!(
        imported.events[0].content.as_deref(),
        Some("Hi from desktop")
    );
}
