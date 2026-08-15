//! Small public configuration enums shared by the universal solver surfaces.

/// Runtime surface where the solver is embedded.
///
/// Self-awareness answers use this to avoid claiming browser-only, CLI-only, or
/// server-only affordances in the wrong environment.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionSurface {
    #[default]
    RustLibrary,
    Cli,
    HttpServer,
    Browser,
    Telegram,
    DockerMicroservice,
}

impl ExecutionSurface {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::RustLibrary => "rust_library",
            Self::Cli => "cli",
            Self::HttpServer => "http_server",
            Self::Browser => "browser",
            Self::Telegram => "telegram",
            Self::DockerMicroservice => "docker_microservice",
        }
    }

    pub(crate) fn from_env_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "rust" | "rust_library" | "library" | "lib" => Some(Self::RustLibrary),
            "cli" | "terminal" | "shell" => Some(Self::Cli),
            "http" | "http_server" | "server" | "api" => Some(Self::HttpServer),
            "browser" | "web" | "wasm" | "demo" => Some(Self::Browser),
            "telegram" | "telegram_bot" | "bot" => Some(Self::Telegram),
            "docker" | "docker_microservice" | "container" => Some(Self::DockerMicroservice),
            _ => None,
        }
    }
}

/// How the composite-program `blueprint` synthesizer
/// turns its annotated recipe template into the program shown to the user.
///
/// Issue #340 asked the engine to "try all directions" of program synthesis and
/// let the user switch between them. A blueprint recipe is stored as an annotated
/// template whose optional sub-tasks (error handling, comments, …) are wrapped in
/// `region:<capability>` markers; every emitted program is a *projection* of that
/// template (never the raw, marker-bearing string — markers are always stripped).
/// This knob selects which projection to emit:
///
/// - [`Composed`](Self::Composed) (default, the most promising direction): the
///   program is assembled from exactly the capabilities the request decomposed
///   into — optional regions whose capability the prompt did not ask for are
///   dropped, and when comments were not requested the documentation is stripped
///   too. The same recipe therefore yields genuinely different programs for
///   different requests, which is the honest, anti-memoization demonstration that
///   the code is composed from the decomposition (`NON-GOALS.md`).
/// - [`Documented`](Self::Documented): always emit the fully documented program
///   with every optional region present, regardless of which sub-tasks the
///   request named. Useful as a stable reference and for users who want the
///   maximal annotated program every time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintComposition {
    /// Project the program from the detected capabilities (default).
    #[default]
    Composed,
    /// Always emit the fully documented program with every region present.
    Documented,
}

impl BlueprintComposition {
    /// Stable slug used in the event log and the demo preference value.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Composed => "composed",
            Self::Documented => "documented",
        }
    }

    /// Parse a configuration value (env var or demo preference). Accepts the
    /// canonical slugs plus a few intuitive aliases; returns `None` for anything
    /// unrecognized so callers keep the default.
    #[must_use]
    pub fn from_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "composed" | "compose" | "projection" | "project" | "decomposed" => {
                Some(Self::Composed)
            }
            "documented" | "document" | "full" | "verbatim" | "curated" => Some(Self::Documented),
            _ => None,
        }
    }
}
