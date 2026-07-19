use std::fs;
use std::path::{Path, PathBuf};

const PAGES: &[&str] = &[
    "agentic-clis.md",
    "modes.md",
    "tools.md",
    "memory.md",
    "server-api.md",
    "output-sessions.md",
    "languages.md",
    "desktop.md",
    "vscode.md",
    "telegram.md",
    "docker.md",
    "browser-demo.md",
    "t3-code.md",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn guide(page: &str) -> String {
    read(root().join("docs/configuration").join(page))
}

fn assert_all(label: &str, text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(text.contains(needle), "{label} is missing {needle:?}");
    }
}

#[test]
fn guide_is_discoverable_and_has_copy_paste_setup_for_each_os() {
    let repository_readme = read(root().join("README.md"));
    assert!(repository_readme.contains("docs/configuration/README.md"));

    let index = guide("README.md");
    for page in PAGES {
        assert!(
            index.contains(page),
            "configuration index does not link {page}"
        );
        assert!(
            root().join("docs/configuration").join(page).is_file(),
            "missing configuration page {page}"
        );
    }
    assert_all(
        "configuration index",
        &index,
        &[
            "macOS",
            "Linux",
            "Windows PowerShell",
            "install.sh",
            "install.ps1",
        ],
    );
}

#[test]
fn agentic_cli_page_covers_every_registry_entry_and_mode() {
    let registry = read(root().join("data/seed/client-integrations.lino"));
    let page = guide("agentic-clis.md");
    let ids: Vec<_> = registry
        .lines()
        .filter_map(|line| line.strip_prefix("  tool \"")?.strip_suffix('"'))
        .collect();
    assert!(!ids.is_empty());
    for id in ids {
        assert!(
            page.contains(&format!("## `{id}`")),
            "agentic CLI guide is out of sync with registered tool {id}"
        );
    }
    assert_all(
        "agentic CLI guide",
        &page,
        &[
            "formal-ai with",
            "--global",
            "--undo",
            "--interactive",
            "--non-interactive",
            "dummy",
            "base URL",
            "model",
            "OpenCode Desktop",
            "opencode-desktop",
            "FORMAL_AI_OPENCODE_DESKTOP_BIN",
            "verify",
        ],
    );
}

#[test]
fn modes_and_tools_are_documented_from_capability_to_fallback() {
    assert_all(
        "modes guide",
        &guide("modes.md"),
        &[
            "out-of-box",
            "passthrough",
            "Agent CLI",
            "agent-commander",
            "engine selector",
            "installed Agent",
            "default",
            "engine",
        ],
    );
    let tools_page = guide("tools.md");
    assert_all(
        "tools guide",
        &tools_page,
        &[
            "Internal tools",
            "External tools",
            "capability",
            "environment",
            "specialized",
            "bash",
            "fallback",
            "hosted",
        ],
    );

    let environments = read(root().join("data/seed/environments.lino"));
    for line in environments
        .lines()
        .filter(|line| line.starts_with("    tools ("))
    {
        for tool in line.split('"').skip(1).step_by(2) {
            assert!(
                tools_page.contains(&format!("`{tool}`")),
                "tools guide is out of sync with environment tool {tool}"
            );
        }
    }
}

#[test]
fn memory_server_output_and_language_contracts_are_documented() {
    assert_all(
        "memory guide",
        &guide("memory.md"),
        &[
            "~/.formal-ai/",
            "%APPDATA%\\formal-ai\\",
            "memory.lino",
            "-v \"$HOME/.formal-ai:/root/.formal-ai\"",
            "Telegram",
            "VS Code",
            "FORMAL_AI_MEMORY_PATH",
        ],
    );
    assert_all(
        "server and API guide",
        &guide("server-api.md"),
        &[
            "--agent-mode",
            "hosted",
            "UTF-8",
            "context_window_tokens",
            "context_used_tokens",
            "context_used_fraction",
            "disk_free_bytes",
            "usage",
            "cost",
        ],
    );
    assert_all(
        "output and sessions guide",
        &guide("output-sessions.md"),
        &[
            "friendly",
            "```json",
            "transcript",
            "session",
            "resume",
            "FORMAL_AI_PROXY_LOG",
        ],
    );
    assert_all(
        "language guide",
        &guide("languages.md"),
        &[
            "data/seed/",
            "data-only",
            "parity",
            "English",
            "Russian",
            "Chinese",
            "Hindi",
        ],
    );
}

#[test]
fn each_requested_surface_has_setup_and_shared_memory_guidance() {
    for (page, markers) in [
        ("desktop.md", &["Install", "engine", "shared memory"][..]),
        (
            "vscode.md",
            &["Install", "desktop host", "web host", "shared memory"],
        ),
        (
            "telegram.md",
            &["TELEGRAM_BOT_TOKEN", "polling", "webhook", "shared memory"],
        ),
        (
            "docker.md",
            &["docker compose", "-v", "/root/.formal-ai", "shared memory"],
        ),
        (
            "browser-demo.md",
            &[
                "GitHub Pages",
                "in-process",
                "Export memory",
                "Import memory",
            ],
        ),
        (
            "t3-code.md",
            &["formal-ai with t3code", "Codex", "Claude", "shared memory"],
        ),
    ] {
        assert_all(page, &guide(page), markers);
    }
}

#[test]
fn whole_configuration_guide_covers_issue_761() {
    let mut complete = guide("README.md");
    for page in PAGES {
        complete.push_str(&guide(page));
    }
    assert_all(
        "whole configuration guide",
        &complete,
        &[
            "formal-ai with cursor",
            "formal-ai with t3code",
            "formal-ai with opencode",
            "out-of-box",
            "agent-commander",
            "specialized",
            "~/.formal-ai/",
            "--agent-mode",
            "context_used_fraction",
            "```json",
            "data-only",
            "Windows PowerShell",
        ],
    );
}
