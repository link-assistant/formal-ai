//! VS Code extension surface tests.
//!
//! Issue #353 requires a VS Code extension whose chat UI supports every feature
//! the web app does, that can spin up the local server and drive Docker code
//! execution from extension settings, and that also runs in browser-based VS Code
//! (vscode.dev / github.dev). Rather than fork the symbolic solver, the extension
//! embeds the committed `src/web/` chat in a Webview and reuses the desktop
//! shell's bridge, tool router, memory-sync and HTTP boundary.
//!
//! These tests pin the contracts that make that true: a dual-host manifest, a
//! Node host that mirrors the Electron shell (server + Docker + permission-gated
//! tools + memory sync), a web host that stays in-process with no `node:*` or
//! process spawning, a Webview HTML builder that reconciles the sandbox, and the
//! shared bridge/config that map settings onto the same `desktopStatus` the web
//! app already understands. The runtime assertions exercise the very HTTP and
//! memory features the Node host depends on, proving the engine is reused.

use formal_ai::{
    environment_directory, export_memory_full, handle_api_request, import_memory_full, seed_files,
    BundleInfo, MemoryEvent, MemoryStore,
};

const VSCODE_PACKAGE: &str = include_str!("../../../vscode/package.json");
const EXTENSION_NODE: &str = include_str!("../../../vscode/src/extension.node.cjs");
const EXTENSION_WEB: &str = include_str!("../../../vscode/src/extension.web.cjs");
const CONFIG_LIB: &str = include_str!("../../../vscode/src/lib/config.cjs");
const BRIDGE_LIB: &str = include_str!("../../../vscode/src/lib/bridge.cjs");
const WEBVIEW_HTML_LIB: &str = include_str!("../../../vscode/src/lib/webview-html.cjs");
const CHAT_VIEW_LIB: &str = include_str!("../../../vscode/src/lib/chat-view.cjs");
const SERVER_PROCESS_LIB: &str = include_str!("../../../vscode/src/lib/server-process.cjs");
const PREPARE_RESOURCES: &str = include_str!("../../../vscode/scripts/prepare-resources.mjs");
const WEB_APP: &str = include_str!("../../../src/web/app.js");

#[test]
fn vscode_manifest_declares_dual_host_commands_and_settings() {
    let manifest: serde_json::Value = serde_json::from_str(VSCODE_PACKAGE).unwrap();
    assert_eq!(manifest["name"], "formal-ai-vscode");

    // Dual host: desktop/remote Node host (`main`) + browser Web Worker host
    // (`browser`), so the extension loads on vscode.dev and github.dev too.
    assert_eq!(manifest["main"], "./src/extension.node.cjs");
    assert_eq!(manifest["browser"], "./src/extension.web.cjs");

    // R6 / web VS Code: the extension must advertise that it works in virtual and
    // untrusted workspaces (the in-process agent is safe everywhere).
    assert_eq!(manifest["capabilities"]["virtualWorkspaces"], true);
    assert_eq!(
        manifest["capabilities"]["untrustedWorkspaces"]["supported"],
        true
    );

    // The chat UI is contributed as a webview view.
    let view = &manifest["contributes"]["views"]["formal-ai"][0];
    assert_eq!(view["type"], "webview");
    assert_eq!(view["id"], "formal-ai.chatView");

    let commands = manifest["contributes"]["commands"]
        .as_array()
        .expect("commands array");
    let command_ids: Vec<&str> = commands
        .iter()
        .filter_map(|command| command["command"].as_str())
        .collect();
    for expected in [
        "formal-ai.openChat",
        "formal-ai.toggleServer",
        "formal-ai.syncMemory",
        "formal-ai.openNetworkView",
    ] {
        assert!(
            command_ids.contains(&expected),
            "manifest must contribute the {expected} command; got {command_ids:?}"
        );
    }

    // Every feature is driven by `formal-ai.*` settings, as the issue requires.
    let properties = manifest["contributes"]["configuration"]["properties"]
        .as_object()
        .expect("configuration properties");
    for expected in [
        "formal-ai.server.enabled",
        "formal-ai.server.host",
        "formal-ai.server.port",
        "formal-ai.docker.image",
        "formal-ai.tools.allowByDefault",
        "formal-ai.agent.defaultOn",
    ] {
        assert!(
            properties.contains_key(expected),
            "manifest must expose the {expected} setting"
        );
    }
    // The local server defaults to off — code execution is opt-in.
    assert_eq!(
        properties["formal-ai.server.enabled"]["default"], false,
        "the local server must be opt-in (default false)"
    );
}

#[test]
fn vscode_version_syncs_from_the_rust_crate() {
    // vscode/package.json carries a committed baseline version; the published
    // .vsix is stamped from Cargo.toml (the single source of truth) by the
    // package step, exactly like desktop/package.json. We pin the *sync
    // mechanism* rather than the committed value: the release pipeline
    // (scripts/version-and-commit.rs) bumps Cargo.toml without touching any
    // package.json, so a strict equality assertion here would break main on
    // the very next release.
    let manifest: serde_json::Value = serde_json::from_str(VSCODE_PACKAGE).unwrap();
    let version = manifest["version"]
        .as_str()
        .expect("vscode/package.json version must be a string");
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())),
        "vscode/package.json version must be semver MAJOR.MINOR.PATCH, got {version:?}"
    );

    // The package step stamps the extension version from Cargo.toml's
    // [package] version so the published listing always matches the crate,
    // even when the committed baseline lags between releases.
    assert!(
        PREPARE_RESOURCES.contains("syncExtensionVersion"),
        "prepare-resources must define syncExtensionVersion"
    );
    assert!(
        PREPARE_RESOURCES.contains("Cargo.toml")
            && PREPARE_RESOURCES.contains("vscodePackage.version = cargoVersion"),
        "syncExtensionVersion must stamp package.json version from Cargo.toml's [package] version"
    );
}

#[test]
fn vscode_node_host_mirrors_the_electron_desktop_shell() {
    // The Node host can spin up the local server, run host shell commands, drive
    // Docker for sandboxed code, route tools behind permission, and reconcile
    // memory: the desktop affordances the issue asks for, all behind
    // `formal-ai.*` settings.
    assert!(
        EXTENSION_NODE.contains(r#"require("node:child_process")"#),
        "the Node host needs child_process to spawn the server and Docker"
    );
    for expected in [
        "startServer",
        "createToolRouter",
        "createMemorySync",
        "dockerIsAvailable",
        "runInSandbox",
        "runOnHost",
        "formal-ai.server.enabled",
        r#"SHELL = "VS Code""#,
    ] {
        assert!(
            EXTENSION_NODE.contains(expected),
            "extension.node.cjs should wire `{expected}`"
        );
    }
    // Code execution is sandboxed in the configured Docker image, never inline.
    assert!(
        EXTENSION_NODE.contains("docker.image"),
        "sandboxed code execution must honour the formal-ai.docker.image setting"
    );
    // Tool routing / memory sync only apply once the local server is the
    // execution surface (otherwise the browser sandbox holds).
    assert!(
        EXTENSION_NODE.contains("status.serverEnabled && status.apiReady"),
        "tool routing must require a healthy local server"
    );
}

#[test]
fn vscode_web_host_is_in_process_only_with_no_node_builtins() {
    // R6 / ROADMAP V5: the web host runs inside a Web Worker on vscode.dev, which
    // cannot spawn a process or talk to Docker. It must therefore avoid every
    // `node:*` builtin and the server / tool-router / memory-sync code, and pin
    // the surface to in-process.
    assert!(EXTENSION_WEB.contains(r#"SHELL = "VS Code Web""#));
    assert!(
        EXTENSION_WEB.contains("serverCapable: false"),
        "the web host must report serverCapable: false"
    );
    for forbidden in [
        r#"require("node:"#,
        "startServer(",
        "createToolRouter(",
        "createMemorySync(",
    ] {
        assert!(
            !EXTENSION_WEB.contains(forbidden),
            "extension.web.cjs must not contain `{forbidden}` (it loads in a Web Worker)"
        );
    }
    // It still renders the same chat UI through the shared provider + bridge.
    assert!(EXTENSION_WEB.contains("createChatViewProvider"));
    assert!(EXTENSION_WEB.contains("createBridge"));
}

#[test]
fn vscode_webview_html_reconciles_the_sandbox() {
    // The Webview HTML builder injects a `<base>` for the resource origin, a
    // strict nonce CSP, the same-origin Worker shim (so the WASM engine loads),
    // and the FormalAiDesktop postMessage bridge — without forking the web app.
    for expected in [
        r"<base href=",
        "Content-Security-Policy",
        "wasm-unsafe-eval",
        "window.Worker = function",
        "window.FormalAiDesktop",
        "formalAiDesktop:request",
        "formalAiDesktop:response",
        "getStatus",
        "setToolGrants",
        "invokeTool",
        "syncMemory",
        "openExternal",
    ] {
        assert!(
            WEBVIEW_HTML_LIB.contains(expected),
            "webview-html.cjs should inject `{expected}`"
        );
    }
    // The chat-view provider exposes both hosts the same localResourceRoots and
    // prefers the packaged dist-web layout, falling back to the dev checkout.
    assert!(CHAT_VIEW_LIB.contains("localResourceRoots"));
    assert!(CHAT_VIEW_LIB.contains("dist-web"));
}

#[test]
fn vscode_bridge_implements_the_desktop_contract_default_deny() {
    // The bridge backs the exact `FormalAiDesktop` contract the desktop preload
    // exposes, host-agnostically, so the web app needs no shell-specific code.
    for expected in [
        "getStatus",
        "setToolGrants",
        "invokeTool",
        "syncMemory",
        "openExternal",
        "dispatch",
    ] {
        assert!(
            BRIDGE_LIB.contains(expected),
            "bridge.cjs should expose `{expected}`"
        );
    }
    // Default-deny: tool calls are refused until the local server is the
    // execution surface, and external links are restricted to http(s).
    assert!(BRIDGE_LIB.contains("requires the local server"));
    assert!(BRIDGE_LIB.contains("isServerEnabled"));
    assert!(BRIDGE_LIB.contains(r"^https?:\/\/"));
}

#[test]
fn vscode_config_maps_settings_to_the_desktop_status_shape() {
    // The config mapper turns `formal-ai.*` settings into the same desktopStatus
    // the web app drives every affordance off, defaulting to in-process.
    for expected in [
        "statusFromConfig",
        "withApiReady",
        "withApiError",
        "serverCapable",
        "formal_ai_bundle",
        r#"mode: serverEnabled ? "server" : "in-process""#,
        "/v1/chat/completions",
        "/v1/graph",
    ] {
        assert!(
            CONFIG_LIB.contains(expected),
            "config.cjs should define `{expected}`"
        );
    }

    // The web app only routes to the local server when it is genuinely ready,
    // otherwise it stays on the in-process engine — the same gate the desktop
    // shell relies on.
    assert!(
        WEB_APP.contains("currentDesktopStatus.apiReady && currentDesktopStatus.apiBase"),
        "app.js must only call the local server when it is ready"
    );
}

#[test]
fn vscode_server_process_launches_formal_ai_serve() {
    // The Node host launches `formal-ai serve` from a binary override, cargo, or
    // PATH, and waits on the /health endpoint before promoting the status.
    for expected in [
        "apiCandidates",
        "waitForApi",
        "requestHealth",
        "/health",
        r#""serve""#,
        "FORMAL_AI_VSCODE_BINARY",
        "cargo",
    ] {
        assert!(
            SERVER_PROCESS_LIB.contains(expected),
            "server-process.cjs should define `{expected}`"
        );
    }
}

#[test]
fn vscode_web_surface_labels_the_host_from_its_shell() {
    // The shared web app labels the surface from the bridge's shell string, so a
    // VS Code status renders "VS Code" while the Electron shell stays "Desktop".
    assert!(
        WEB_APP.contains("function desktopSurfaceLabel(status)"),
        "app.js must derive the surface label from the status shell"
    );
    assert!(
        WEB_APP.contains(
            r#"/code/i.test(String((status && status.shell) || "")) ? "VS Code" : "Desktop""#
        ),
        "app.js must map a code-shell status to the VS Code label and keep Desktop otherwise"
    );
}

#[test]
fn vscode_environment_is_declared_in_seed_directory() {
    let directory = environment_directory();
    let vscode = directory
        .environments
        .iter()
        .find(|environment| environment.id == "vscode")
        .expect("vscode environment should be declared in seed directory");
    let searchable = [
        vscode.label.as_str(),
        vscode.runtime.as_str(),
        vscode.seed_path.as_str(),
        vscode.memory_store.as_str(),
        vscode.memory_export_command.as_str(),
        vscode.bundle_export_command.as_str(),
        vscode.bundle_import_command.as_str(),
        vscode.start_command.as_str(),
        vscode.package_command.as_str(),
        &vscode.tools.join("|"),
    ]
    .join("\n");
    for expected in [
        "VS Code extension (desktop + web)",
        "Node extension host",
        "Web Worker host",
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
    for expected_flow in ["browser_to_vscode", "vscode_local_sync"] {
        assert!(
            directory.flows.iter().any(|flow| flow.id == expected_flow),
            "environment seed should mention flow `{expected_flow}`"
        );
    }
}

#[test]
fn vscode_chat_path_reuses_openai_http_completion_endpoint() {
    // The Node host routes chat to the very same /v1/chat/completions the web and
    // desktop surfaces use, so the feature set is identical.
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
fn vscode_network_view_reuses_graph_endpoint() {
    // The "Open Network View" command surfaces the shared /v1/graph endpoint.
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
fn vscode_memory_import_export_round_trips_full_bundle() {
    // Memory import/export keeps the shared full-bundle format, so a conversation
    // started in the VS Code webview is portable to every other surface.
    let mut store = MemoryStore::new();
    store.append(MemoryEvent::user("Hi from VS Code"));
    store.append(MemoryEvent::assistant("Hi, how may I help you?"));

    let bundle = export_memory_full(&seed_files(), store.events(), &[], &BundleInfo::default());
    assert!(bundle.starts_with("formal_ai_bundle"));
    let imported = import_memory_full(&bundle);

    assert_eq!(imported.events.len(), 2);
    assert_eq!(
        imported.events[0].content.as_deref(),
        Some("Hi from VS Code")
    );
}
