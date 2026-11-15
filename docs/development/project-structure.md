# Project structure

The top-level layout of the repository. This was moved out of
[CONTRIBUTING.md](../../CONTRIBUTING.md#project-structure) to keep that file
under its 1,500-line limit (issue #1191).

```
.
├── .github/workflows/    # GitHub Actions CI/CD
├── analysis/             # Ad-hoc analysis notes
├── changelog.d/          # Changelog fragments
│   ├── README.md         # Fragment instructions
│   └── *.md              # Individual changelog fragments
├── data/                 # Canonical knowledge surface (Links Notation)
│   ├── seed/             # Seed data every interface reads (see "Data is the interface")
│   ├── meta/             # Grounded meta-algorithm recipes and lexicons
│   ├── benchmarks/       # Benchmark suites and license provenance
│   ├── cache/            # Precached external sources (Wikidata, …)
│   └── parity/           # Cross-runtime parity fixtures
├── desktop/              # Electron desktop shell
├── docs/                 # Case studies, design notes, diagrams, guides
│   ├── requirements-traceability.md  # One delivery row per requirement ID
│   ├── benchmarks.md     # Every benchmark suite, its license, and honest scores
│   ├── meta-algorithm.md # The grounded meta-algorithm and its procedures
│   └── architect-notes/  # Dated notes from the architect, indexed in README.md
├── rust/examples/             # Usage examples
├── experiments/          # Verification harnesses and one-off experiments
├── scripts/              # Rust scripts (via rust-script) and installers
├── rust/src/
│   ├── lib.rs            # Library entry point
│   ├── main.rs           # Binary entry point
│   ├── solver.rs         # UniversalSolver — the 11-step loop
│   ├── solver_handlers/  # Handler family invoked through the method registry
│   ├── agentic_coding/   # Agentic-CLI planner, recipes, and driver
│   ├── proof_engine/     # Decision procedures
│   ├── summarization/    # Formalize → summarize → deformalize pipeline
│   ├── translation/      # Formalization and translation through meanings
│   ├── seed/             # Seed loading, lexicon, and role constants
│   └── web/              # Browser demo: UI, worker/, and wasm-worker/
├── rust/tests/                # Unit, integration, source-mirror, and e2e tests
├── vscode/               # VS Code extension (desktop and web)
├── .gitignore            # Git ignore patterns
├── .pre-commit-config.yaml  # Pre-commit hooks
├── AGENTS.md             # Rules for AI agents: the single push gate
├── ARCHITECTURE.md       # How the implemented pipeline is wired
├── build.rs              # Build script
├── Cargo.toml            # Project configuration
├── CHANGELOG.md          # Project changelog
├── CLAUDE.md             # Points Claude Code at AGENTS.md
├── compose.yaml          # Docker Compose profiles
├── CONTRIBUTING.md       # How to contribute
├── Dockerfile            # Docker-in-Docker Telegram runtime
├── GOALS.md              # What counts as success
├── LICENSE               # Unlicense (public domain)
├── NON-GOALS.md          # What we explicitly do not build
├── package.json          # Web/desktop/VS Code tooling scripts
├── README.md             # Project README
├── REQUIREMENTS.md       # Issue-by-issue requirement matrix
├── ROADMAP.md            # Requirement-level implementation status
└── VISION.md             # Standing guideline and long-term direction; read first
```
