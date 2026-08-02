#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for the first issue #708 test leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-708/self-hosting-authorship"
CANONICAL="$ROOT/tests/unit/issue_708_memory_program.rs"
TASK='Create a Rust regression test file named issue_708_memory_program.rs with exactly this content:
use formal_ai::memory_program::{compile_memory_program, MemoryProgramLimits};

#[test]
fn equivalent_multilingual_requests_compile_to_the_same_program_links() {
    let requests = [
        "List every fact I contributed about X and rename X to Y in all of them.",
        "Перечисли все факты, которые я добавил о X, и переименуй X в Y во всех них.",
        "X के बारे में मेरे जोड़े हर तथ्य को सूचीबद्ध करो और उन सभी में X का नाम Y कर दो।",
        "列出我贡献的关于 X 的每个事实，并在所有事实中将 X 重命名为 Y。",
    ];
    let limits = MemoryProgramLimits {
        max_matches: 32,
        max_iterations: 4,
    };
    let programs = requests
        .iter()
        .map(|request| compile_memory_program(request, limits).expect("request should compile"))
        .collect::<Vec<_>>();

    for program in &programs[1..] {
        assert_eq!(program.id, programs[0].id);
        assert_eq!(program.links_notation(), programs[0].links_notation());
    }
    assert_eq!(
        programs[0].primitive_names(),
        [
            "match",
            "filter",
            "map_matches",
            "update",
            "sequential_compose",
            "bounded_iterate_to_fixpoint",
        ]
    );
    assert!(programs[0].links_notation().contains("max_matches 32"));
    assert!(programs[0].links_notation().contains("max_iterations 4"));
}'

TASK="$TASK" \
EXPECT_FILE="issue_708_memory_program.rs" \
EXPECT_TEXT="equivalent_multilingual_requests_compile_to_the_same_program_links" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8708}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/issue_708_memory_program.rs" "$CANONICAL"
cmp "$ARTIFACT_DIR/issue_708_memory_program.rs" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
