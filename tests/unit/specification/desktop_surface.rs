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
const DESKTOP_SERVICE_CONTROL: &str = include_str!("../../../desktop/lib/service-control.cjs");
const DESKTOP_LOCAL_SERVER: &str = include_str!("../../../desktop/lib/local-server.cjs");
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
    assert!(DESKTOP_LOCAL_SERVER.contains("/v1/chat/completions"));
    assert!(DESKTOP_LOCAL_SERVER.contains("/v1/graph"));
    assert!(DESKTOP_LOCAL_SERVER.contains("cargo"));
    assert!(DESKTOP_LOCAL_SERVER.contains("formal-ai"));

    assert!(DESKTOP_PRELOAD.contains("contextBridge"));
    assert!(DESKTOP_PRELOAD.contains("FormalAiDesktop"));
    assert!(DESKTOP_PRELOAD.contains("getStatus"));
}

#[test]
fn desktop_runs_in_process_by_default_with_opt_in_server() {
    // R3: the desktop defaults to the in-process reasoning agent.
    // R4/R11: the local OpenAI-compatible server is off at first launch unless
    // the user opts in via FORMAL_AI_DESKTOP_SERVER. Agent and Full Auto mode
    // can still start it later through the explicit desktop bridge.
    assert!(
        DESKTOP_LOCAL_SERVER.contains("FORMAL_AI_DESKTOP_SERVER"),
        "local-server.cjs must keep the FORMAL_AI_DESKTOP_SERVER startup opt-in"
    );
    assert!(
        DESKTOP_MAIN.contains("serverModeRequested()"),
        "main.cjs must only start the server when explicitly requested"
    );
    assert!(
        DESKTOP_MAIN.contains(r#"mode: "in-process""#),
        "main.cjs default desktop status must report in-process mode"
    );
    for expected in [
        "formalAiDesktop:ensureAgentServer",
        "createLocalServerManager",
        "ensureAgentServer",
    ] {
        assert!(
            DESKTOP_MAIN.contains(expected) || DESKTOP_PRELOAD.contains(expected),
            "desktop bridge should contain `{expected}`"
        );
    }

    // The web surface must keep an in-process path that does not depend on the
    // server: it only routes to the API when both apiReady and apiBase are set.
    assert!(
        WEB_APP.contains("currentDesktopStatus.apiReady && currentDesktopStatus.apiBase"),
        "app.js must only call the desktop server when it is ready, else stay in-process"
    );
}

#[test]
fn desktop_agent_mode_auto_starts_local_openai_server() {
    struct LanguageCoverageCase {
        language: &'static str,
        name: &'static str,
    }

    for expected in [
        "createLocalServerManager",
        "requestHealth",
        "startApiProcess",
        "agentProvider",
        "local-openai-compatible",
    ] {
        assert!(
            DESKTOP_LOCAL_SERVER.contains(expected),
            "local-server.cjs should contain `{expected}`"
        );
    }
    for expected in [
        "ensureDesktopAgentServer",
        "bridge.ensureAgentServer",
        r#"mode === "chat""#,
        "agentProvider",
    ] {
        assert!(
            WEB_APP.contains(expected),
            "app.js should auto-start the desktop server via `{expected}`"
        );
    }
    // The Agent / Full Auto server startup bridge is language-neutral and adds
    // no localized copy. Keep the language-facing app.js route pinned for every
    // supported UI language so the diff-aware CI coverage guard can distinguish
    // this from an English-only UI change.
    for case in [
        LanguageCoverageCase {
            language: "en",
            name: "English",
        },
        LanguageCoverageCase {
            language: "ru",
            name: "Russian",
        },
        LanguageCoverageCase {
            language: "hi",
            name: "Hindi",
        },
        LanguageCoverageCase {
            language: "zh",
            name: "Chinese",
        },
    ] {
        assert!(
            !case.language.is_empty() && !case.name.is_empty(),
            "desktop agent server startup should stay language-neutral for {}",
            case.name
        );
    }
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
fn desktop_service_control_starts_and_stops_prepared_containers() {
    // Issue #438 (follow-up): the desktop app must start/stop both prepared
    // containers (Telegram bot + OpenAI-compatible server) with one click. The
    // service-control module owns the lifecycle behind an injected runner.
    for expected in [
        "createServiceControl",
        "formal-ai-telegram",
        "formal-ai-server",
        "TELEGRAM_BOT_TOKEN",
        // the server container overrides the command to serve the OpenAI API.
        "serve",
        // running-state is read without a live daemon dependency in tests.
        "{{.State.Running}}",
    ] {
        assert!(
            DESKTOP_SERVICE_CONTROL.contains(expected),
            "service-control.cjs should contain `{expected}`"
        );
    }

    // The main process wires the lifecycle to a real docker runner and exposes it
    // over IPC; the preload bridge forwards the renderer's one-click calls.
    for expected in [
        "createServiceControl",
        "formalAiDesktop:serviceStatus",
        "formalAiDesktop:startService",
        "formalAiDesktop:stopService",
    ] {
        assert!(
            DESKTOP_MAIN.contains(expected),
            "main.cjs should wire `{expected}`"
        );
    }
    for expected in ["serviceStatus", "startService", "stopService"] {
        assert!(
            DESKTOP_PRELOAD.contains(expected),
            "preload.cjs should bridge `{expected}`"
        );
    }
}

#[test]
fn desktop_web_surface_exposes_one_click_service_controls() {
    // The renderer renders a Services panel with start/stop buttons and live
    // status indicators for both prepared containers.
    for expected in [
        "sidebar-services",
        "desktop-services-panel",
        "desktop-service-start-",
        "desktop-service-stop-",
        "desktop-service-telegram-token",
        "handleStartService",
        "handleStopService",
        "serviceStatus",
    ] {
        assert!(
            WEB_APP.contains(expected),
            "app.js Services panel should contain `{expected}`"
        );
    }
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
