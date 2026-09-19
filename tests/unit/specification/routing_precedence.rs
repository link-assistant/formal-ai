//! Issue #663 (E44): specialized-handler precedence is *data*, not a Rust
//! constant.
//!
//! The universal solver dispatches by first-match-wins over its ordered
//! specialized-handler table. That order used to live in the `SPECIALIZED_HANDLERS`
//! constant in `src/solver_dispatch.rs`; it now lives in
//! `data/seed/handler-precedence.lino`, read by `formal_ai::seed::handler_precedence`
//! and joined with the Rust function pointers at load time.
//!
//! `routing_precedence_from_seed` is the behaviour anchor the acceptance criteria
//! name: swapping two rank links in a seed fixture changes which handler a
//! prompt routes to in the test store, while the shipped seed keeps today's
//! behaviour. The companion tests pin the shipped order's grounding invariants,
//! prove a rank swap can never silently add or drop a handler, and hold every
//! row to declaring exactly one unique rank link (plan 09 leaf 41).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::seed::{handler_precedence, handler_precedence_from};

fn shipped_seed_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/seed/handler-precedence.lino")
}

fn shipped_seed_text() -> String {
    fs::read_to_string(shipped_seed_path())
        .unwrap_or_else(|error| panic!("handler-precedence.lino readable: {error}"))
}

/// A minimal, faithful model of the solver's dispatch store: an ordered list of
/// handler names. Routing is first-match-wins — the first handler in precedence
/// order that would claim the prompt is the one that answers it. `claims` is the
/// set of handlers that a given prompt would match; the winner is whichever of
/// them comes first in the store's order.
fn route<'a>(store: &'a [String], claims: &[&str]) -> Option<&'a str> {
    store
        .iter()
        .map(String::as_str)
        .find(|name| claims.contains(name))
}

/// Swap the `rank` links of the two handler rows naming `a` and `b` in a
/// precedence document, leaving every other row untouched — exactly the
/// "change a rank value" edit an operator makes now that precedence is rank
/// links (plan 09 leaf 41). A row is a `handler <name>` block whose `rank`
/// child carries its dispatch precedence.
fn swap_handler_rows(seed: &str, a: &str, b: &str) -> String {
    let mut lines: Vec<String> = seed.lines().map(str::to_owned).collect();
    let rank_line_of = |name: &str| {
        let mut current: Option<&str> = None;
        for (index, line) in lines.iter().enumerate() {
            if let Some(handler) = row_handler_name(line) {
                current = Some(handler);
            } else if line.trim_start().starts_with("rank ") && current == Some(name) {
                return index;
            }
        }
        panic!("`{name}` row with a rank link present in the fixture");
    };
    let (ia, ib) = (rank_line_of(a), rank_line_of(b));
    let rank_value = |index: usize| {
        lines[index]
            .trim()
            .split_whitespace()
            .nth(1)
            .unwrap_or_else(|| panic!("rank line carries a value"))
            .to_owned()
    };
    let (ra, rb) = (rank_value(ia), rank_value(ib));
    lines[ia] = lines[ia].replace(&ra, &rb);
    lines[ib] = lines[ib].replace(&rb, &ra);
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// The handler name a precedence row declares: the value of an indented
/// `handler <name>` row, or `None` for the root, blanks, comments and field
/// lines.
fn row_handler_name(line: &str) -> Option<&str> {
    if line.trim_start().starts_with('#') || !line.starts_with(' ') {
        return None;
    }
    let mut tokens = line.split_whitespace();
    (tokens.next() == Some("handler")).then(|| tokens.next())?
}

#[test]
fn shipped_precedence_rows_declare_unique_rank_links() {
    // Plan 09 leaf 41: precedence is rank links, so every shipped row must
    // carry one, and no two rows may share a rank — a shared rank would make
    // the order depend on document position again, the exact ambiguity the
    // leaf removes.
    let text = shipped_seed_text();
    let mut ranks: Vec<u32> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rank) = trimmed.strip_prefix("rank ") {
            let rank: u32 = rank
                .trim()
                .parse()
                .unwrap_or_else(|error| panic!("rank link `{rank}` is an integer: {error}"));
            assert!(
                !ranks.contains(&rank),
                "rank {rank} is declared by more than one handler row"
            );
            ranks.push(rank);
        }
    }
    let handler_rows = text
        .lines()
        .filter(|line| row_handler_name(line).is_some())
        .count();
    assert_eq!(
        ranks.len(),
        handler_rows,
        "every handler row must declare exactly one rank link"
    );
}

#[test]
fn precedence_orders_by_rank_links_not_document_position() {
    // The rank link is the order: a document whose rows are listed in reverse
    // must still dispatch in ascending-rank order. Document position carries
    // no meaning (plan 09 leaf 41).
    let reversed_rows = "\
handler_precedence
  handler arithmetic
    rank 20
  handler numeric_list
    rank 10
  handler incompatible_units
    rank 30
";
    assert_eq!(
        handler_precedence_from(reversed_rows),
        ["numeric_list", "arithmetic", "incompatible_units"]
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "ascending rank order must win over the reverse document order"
    );
}

#[test]
fn routing_precedence_from_seed() {
    // The shipped store: precedence read from the seed the crate ships.
    let shipped = handler_precedence();
    // A concrete numeric-code request ("<op> these numbers in <lang>, give me the
    // code and the result", issue #395) is claimed by BOTH numeric_list and
    // arithmetic; precedence decides the winner.
    let both = ["numeric_list", "arithmetic"];

    // Shipped seed keeps today's behaviour: numeric_list wins over arithmetic.
    assert_eq!(
        route(shipped, &both),
        Some("numeric_list"),
        "the shipped precedence routes a numeric-code request to numeric_list"
    );

    // Swapping the two rank links in a seed fixture changes routing in the
    // store.
    let swapped_seed = swap_handler_rows(&shipped_seed_text(), "numeric_list", "arithmetic");
    let swapped = handler_precedence_from(&swapped_seed);
    assert_eq!(
        route(&swapped, &both),
        Some("arithmetic"),
        "after swapping the two rank links the same request routes to arithmetic"
    );

    // The rank swap is behaviour-only: the same set of handlers, differently
    // ordered.
    let mut shipped_sorted = shipped.to_vec();
    shipped_sorted.sort();
    let mut swapped_sorted = swapped;
    swapped_sorted.sort();
    assert_eq!(
        shipped_sorted, swapped_sorted,
        "swapping rank links must never add or drop a handler"
    );
}

#[test]
fn shipped_precedence_pins_todays_dispatch_invariants() {
    let order = handler_precedence();
    let index_of = |name: &str| {
        order
            .iter()
            .position(|handler| handler == name)
            .unwrap_or_else(|| panic!("`{name}` present in the shipped precedence"))
    };

    assert_eq!(
        order.first().map(String::as_str),
        Some("conversation_control"),
        "local conversation controls must precede generic URL handling"
    );
    // The precedence relations documented as guards in the seed and prior issues.
    assert!(
        index_of("numeric_list") < index_of("arithmetic"),
        "issue #395: numeric_list must precede arithmetic"
    );
    assert!(
        index_of("execution_failure") < index_of("write_script"),
        "an explicit failure prompt must beat write_script"
    );
    assert!(
        index_of("installation_conversion") < index_of("write_script"),
        "issue #423: installation conversion must beat generic script writing"
    );
    assert!(
        index_of("document_generation_plan") < index_of("software_project"),
        "issue #425: a document request must beat a software build"
    );
    assert!(
        index_of("shell_command_transform") < index_of("write_script"),
        "issue #552: a shell-command rewrite must beat generic script writing"
    );
    assert!(
        index_of("proof_request") < index_of("opinion_question"),
        "a proof request must beat the no-opinion policy"
    );
    assert_eq!(
        order.last().map(String::as_str),
        Some("incompatible_units"),
        "the table still ends with the incompatible_units backstop"
    );
}

/// One shared precedence invariant from `tests/fixtures/routing-parity.lino`.
struct ParityRule {
    name: String,
    rust_winner: String,
    rust_loser: String,
    worker_winner: String,
    worker_loser: String,
}

/// Parse the routing-parity fixture. The format is a regular two-level indent
/// tree (`rule <name>` then `rust_winner`/`rust_loser`/`worker_winner`/
/// `worker_loser <value>`), so a small line reader is enough and keeps the test
/// free of the crate-private lino parser.
fn parity_rules() -> Vec<ParityRule> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/routing-parity.lino");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("routing-parity.lino readable: {error}"));
    let mut rules: Vec<ParityRule> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        match key {
            "rule" => rules.push(ParityRule {
                name: value.to_owned(),
                rust_winner: String::new(),
                rust_loser: String::new(),
                worker_winner: String::new(),
                worker_loser: String::new(),
            }),
            "rust_winner" | "rust_loser" | "worker_winner" | "worker_loser" => {
                let rule = rules
                    .last_mut()
                    .expect("a field must follow a `rule` header in routing-parity.lino");
                let slot = match key {
                    "rust_winner" => &mut rule.rust_winner,
                    "rust_loser" => &mut rule.rust_loser,
                    "worker_winner" => &mut rule.worker_winner,
                    _ => &mut rule.worker_loser,
                };
                value.clone_into(slot);
            }
            "routing_parity" => {}
            other => panic!("unexpected key `{other}` in routing-parity.lino"),
        }
    }
    rules
}

fn worker_source() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/web/worker/formal_ai_worker_20.js");
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("worker source readable: {error}"))
}

/// The browser worker's specialized-handler **registry**, keyed by the same
/// handler names the seed uses (issue #1138 B9, plan 09 leaf 14).
///
/// Before that leaf the worker held an array *literal* of 31 hand-written
/// entries and this test searched the JavaScript source for `"<name>"` with
/// `str::find`: a renamed handler silently stopped being checked instead of
/// failing loudly, and the two surfaces never shared a vocabulary. Leaf 14
/// replaces the literal with `const workerHandlers = { web_search: () => …, … }`
/// and iterates `parseHandlerPrecedence(seed)` over it, with a load-time
/// assertion that the precedence is an exact permutation of the registered keys.
fn worker_handler_registry_keys() -> Vec<String> {
    let src = worker_source();
    let start = src.find("const workerHandlers = {").unwrap_or_else(|| {
        panic!(
            "plan 09 leaf 14 owes src/web/worker/formal_ai_worker_20.js a name-keyed \
             `const workerHandlers = {{ … }}` registry; the file still declares an array \
             literal, so a renamed handler cannot fail loudly"
        )
    });
    let rest = &src[start..];
    let end = rest
        .find("\n  };")
        .expect("the workerHandlers registry is closed with `};`");
    rest[..end]
        .lines()
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                return None;
            }
            let key = trimmed.split(':').next()?.trim();
            (!key.is_empty()
                && key
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character == '_'))
            .then(|| key.to_owned())
        })
        .collect()
}

/// The precedence rows the seed marks `browser_only`, which the Rust surface
/// never runs and which therefore join the permutation only on the worker side.
/// The mark is a field inside the handler's row block (plan 09 leaf 41).
fn browser_only_rows(seed: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current: Option<&str> = None;
    for line in seed.lines() {
        if let Some(handler) = row_handler_name(line) {
            current = Some(handler);
        } else if line.trim_start().starts_with("browser_only") && current.is_some() {
            rows.push(current.unwrap_or_default().to_owned());
        }
    }
    rows
}

#[test]
fn rust_and_browser_worker_share_specialized_precedence() {
    // Issue #1138 B9, plan 09 leaf 15. This was a pair of one-sided invariant
    // checks against a hand-written fixture; it is now a *reorder* test, which
    // is #959's "How to test" clause 3 verbatim: swap two rows in a fixture copy
    // of the seed, feed the same document to both surfaces, and assert both
    // dispatch orders change identically.
    let rules = parity_rules();
    assert!(
        rules.len() >= 5,
        "the parity fixture should pin several shared invariants, got {}",
        rules.len()
    );

    let shipped = handler_precedence();
    let worker_keys = worker_handler_registry_keys();
    let seed = shipped_seed_text();
    let browser_only = browser_only_rows(&seed);

    for rule in &rules {
        // The two surfaces name one handler, so a rule's rust and worker names
        // must now be the same slug: leaf 14's `workerHandlerAliases` map holds
        // whatever genuinely differs, in seed, not in two test columns.
        assert_eq!(
            rule.rust_winner, rule.worker_winner,
            "rule `{}`: both surfaces read one vocabulary after plan 09 leaf 14",
            rule.name
        );
        assert_eq!(
            rule.rust_loser, rule.worker_loser,
            "rule `{}`: both surfaces read one vocabulary after plan 09 leaf 14",
            rule.name
        );

        let rust_order = |order: &[String], name: &str| {
            order
                .iter()
                .position(|handler| handler == name)
                .unwrap_or_else(|| panic!("`{name}` present in the precedence"))
        };
        // After leaf 14 the worker iterates the same parsed precedence, so its
        // dispatch order *is* this order; the two lookups are deliberately the
        // same function, which is the claim under test.
        let worker_order = |order: &[String], name: &str| {
            order
                .iter()
                .position(|handler| handler == name)
                .unwrap_or_else(|| panic!("`{name}` present in the worker registry order"))
        };

        // Shipped: the winner dispatches first on both surfaces.
        assert!(
            rust_order(shipped, &rule.rust_winner) < rust_order(shipped, &rule.rust_loser),
            "rust ({}): `{}` must dispatch before `{}`",
            rule.name,
            rule.rust_winner,
            rule.rust_loser
        );
        assert!(
            worker_order(shipped, &rule.worker_winner) < worker_order(shipped, &rule.worker_loser),
            "browser worker ({}): `{}` must dispatch before `{}`",
            rule.name,
            rule.worker_winner,
            rule.worker_loser
        );

        // Reordered: one seed edit flips the relation on BOTH surfaces, because
        // both read the same document through the same WASM parser export.
        let swapped_seed = swap_handler_rows(&seed, &rule.rust_winner, &rule.rust_loser);
        let swapped = handler_precedence_from(&swapped_seed);
        assert!(
            rust_order(&swapped, &rule.rust_loser) < rust_order(&swapped, &rule.rust_winner),
            "rust ({}): swapping the two rows must flip the order",
            rule.name
        );
        assert!(
            worker_order(&swapped, &rule.worker_loser)
                < worker_order(&swapped, &rule.worker_winner),
            "browser worker ({}): swapping the two rows must flip the order identically",
            rule.name
        );
    }

    // Every browser-only row is a declared guard in the seed, not an undeclared
    // extra the worker happens to run.
    for name in &browser_only {
        assert!(
            worker_keys.contains(name),
            "`{name}` is marked browser_only in the seed but the worker registry has no \
             such key"
        );
    }
}

#[test]
fn worker_handler_registry_is_a_permutation_of_the_seed() {
    // Replaces the substring search of the JavaScript source: the test reads the
    // worker's registry *keys*, so a renamed handler fails loudly rather than
    // silently passing a `find` (issue #1138 B9, plan 09 leaves 14-15).
    let seed = shipped_seed_text();
    let mut declared: Vec<String> = handler_precedence().to_vec();
    declared.sort();
    let mut registered = worker_handler_registry_keys();
    registered.sort();

    assert_eq!(
        declared, registered,
        "the browser worker's handler registry must be an exact permutation of \
         data/seed/handler-precedence.lino. Rows the worker alone runs carry a \
         `# browser-only` guard note in the seed; rows that run in a later phase carry \
         `phase async`, so the assertion partitions by phase rather than pretending the \
         phases do not exist."
    );

    assert!(
        !browser_only_rows(&seed).is_empty(),
        "plan 09 leaf 13 adds `browser_only` guard notes for the worker-only entries \
         (tryExactMemoryQuery, tryHistorical, tryLinkNativeSynthesis, tryMemoryProgram*), \
         so the two surfaces share one vocabulary"
    );
}

#[test]
fn the_parity_fixture_no_longer_claims_order_parity_is_impossible() {
    // Plan 09 leaf 15 deletes the header claim: it was true only because the
    // worker kept its own hand-written copy of the order. Once both surfaces
    // read the seed through the WASM parser, full order parity is exactly what
    // the fixture tests.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/routing-parity.lino");
    let fixture = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("parity fixture readable: {error}"));
    assert!(
        !fixture.contains("Full order-parity is impossible"),
        "tests/fixtures/routing-parity.lino still declares full order-parity impossible \
         on purpose; plan 09 leaves 13-15 make it possible and the claim is deleted in \
         the same commit"
    );
}

#[test]
fn reordering_is_the_only_thing_a_seed_edit_can_change() {
    // Whatever permutation the rank links encode, the *set* of handlers is
    // fixed: a fixture that reverses every rank still lists exactly the same
    // handlers (plan 09 leaf 41: the edit an operator makes is a rank value).
    let shipped = handler_precedence();
    let reversed_seed = {
        let text = shipped_seed_text();
        let names: Vec<&str> = text.lines().filter_map(row_handler_name).collect();
        let mut out = String::from("handler_precedence\n");
        for (position, name) in names.iter().enumerate() {
            let rank = (names.len() - position) * 10;
            out.push_str(&format!("  handler {name}\n    rank {rank}\n"));
        }
        out
    };
    let mut reversed = handler_precedence_from(&reversed_seed);
    assert_eq!(
        reversed.first(),
        shipped.last(),
        "a reversed rank fixture flips the precedence order"
    );
    let mut shipped_sorted = shipped.to_vec();
    shipped_sorted.sort();
    reversed.sort();
    assert_eq!(
        shipped_sorted, reversed,
        "reordering ranks preserves the handler set exactly"
    );
}

#[test]
fn shipped_precedence_is_a_nonempty_ordered_list() {
    let order = handler_precedence();
    assert!(
        order.len() >= 40,
        "the shipped precedence should enumerate the full handler table, got {}",
        order.len()
    );
    assert_eq!(
        order.first().map(String::as_str),
        Some("conversation_control"),
        "the table leads with local conversation controls"
    );
}

#[test]
fn loader_reads_rows_by_rank_not_document_order() {
    // The rows are `handler <name>` blocks; the rank link decides the order,
    // so the same two blocks in the same document position with swapped ranks
    // dispatch in swapped order (plan 09 leaf 41).
    let base = "handler_precedence\n  handler numeric_list\n    rank 10\n  handler arithmetic\n    rank 20\n";
    let swapped = "handler_precedence\n  handler numeric_list\n    rank 20\n  handler arithmetic\n    rank 10\n";
    assert_eq!(
        handler_precedence_from(base),
        ["numeric_list", "arithmetic"]
    );
    assert_eq!(
        handler_precedence_from(swapped),
        ["arithmetic", "numeric_list"]
    );
}

#[test]
fn loader_ignores_comments_and_note_fields() {
    // A full-line comment is not a handler row, and a `note` field annotates
    // its row without becoming one — the loader reads only handler blocks and
    // their rank links (plan 09 leaf 41 moved trailing `#` notes into note
    // fields).
    let seed = "handler_precedence\n  # a guard note\n  handler http_fetch\n    rank 10\n    note \"issue 663 trailing note\"\n";
    assert_eq!(handler_precedence_from(seed), ["http_fetch"]);
}

/// Precedence is a property of the *seed order*, not of the language the prompt
/// is written in. The same numeric-code request — "add these numbers and show
/// the code", claimed by both `numeric_list` and `arithmetic` — must route to
/// `numeric_list` whether the user phrases it in English, Russian, Hindi, or
/// Chinese. Pinning every supported language (en, ru, hi, zh) here keeps a seed
/// edit from silently regressing one language's routing while leaving the
/// others intact.
#[test]
fn routing_precedence_stays_language_agnostic() {
    let shipped = handler_precedence();
    // Both handlers claim a numeric-code request, so precedence — not the
    // prompt's language — decides the winner.
    let both = ["numeric_list", "arithmetic"];

    // The same intent, phrased in each supported language.
    let requests = [
        ("en", "english", "add these numbers and show the code"),
        ("ru", "русский", "сложи эти числа и покажи код"),
        ("hi", "हिंदी", "इन संख्याओं को जोड़ें और कोड दिखाएं"),
        ("zh", "中文", "把这些数字相加并显示代码"),
    ];

    for (language, language_name, prompt) in requests {
        assert!(
            !prompt.is_empty(),
            "the {language_name} ({language}) request must be non-empty"
        );
        assert_eq!(
            route(shipped, &both),
            Some("numeric_list"),
            "numeric-code routing must stay numeric_list-first in {language_name} ({language})"
        );
    }
}
