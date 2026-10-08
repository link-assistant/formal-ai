//! Issue #1175 R1175-4: a held-out routing probe corpus over every intent lane.
//!
//! `tests/fixtures/routing-probes/probes-*.lino` holds the probes (prompt,
//! language, lane, the intent the system SHOULD answer) and `budget.lino` the
//! per-lane coverage budget and the misroute ratchets. The browser worker runs
//! the same corpus in `tests/web/r1175-routing-probes.test.mjs`.
//!
//! Here every probe runs through an offline native solver (the worker test
//! stubs every external provider the same way), and every answer is measured.
//! A probe without a `rust_misroute` line must answer its intent — or its
//! `rust_intent`, the designed runtime difference where the native chat
//! surface refuses tool execution outside Agent mode. A probe with a
//! `rust_misroute` line must answer exactly the intent that line names (issue
//! #1173 R1173-3): a misroute is a measured fact, never `unverified`, so a
//! fixed probe fails until its line is dropped and a moved one fails until its
//! line is updated, exactly as the browser test pins `js_misroute`. The number
//! of such lines must equal `rust_misroute_ceiling`, so the set only shrinks.
//! A failure prints the corrected fixture line of every probe that moved.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::{SolverConfig, UniversalSolver};

const LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

#[derive(Debug, Default)]
struct Probe {
    id: String,
    fields: BTreeMap<String, String>,
}

impl Probe {
    fn field(&self, key: &str) -> &str {
        self.fields.get(key).map_or("", String::as_str)
    }

    fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// The intent the native surface must answer.
    fn native_intent(&self) -> &str {
        if self.has("rust_intent") {
            self.field("rust_intent")
        } else {
            self.field("intent")
        }
    }
}

#[derive(Debug, Default)]
struct Budget {
    numbers: BTreeMap<String, usize>,
    extra_lanes: BTreeMap<String, String>,
    exempt_lanes: BTreeMap<String, String>,
}

impl Budget {
    fn number(&self, key: &str) -> usize {
        *self
            .numbers
            .get(key)
            .unwrap_or_else(|| panic!("budget.lino lacks `{key}`"))
    }
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/routing-probes")
}

/// Decode a quoted fixture value: a doubled quote is one quote, `\n` a newline.
fn unquote(raw: &str) -> String {
    let Some(body) = raw
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return raw.to_owned();
    };
    let mut out = String::new();
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, chars.peek()) {
            ('"', Some('"')) => {
                out.push('"');
                chars.next();
            }
            ('\\', Some('n')) => {
                out.push('\n');
                chars.next();
            }
            _ => out.push(ch),
        }
    }
    out
}

/// `(depth, key, value)` for every non-comment line of a fixture file.
fn fixture_lines(path: &Path) -> Vec<(usize, String, String)> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} readable: {error}", path.display()));
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let depth = (line.len() - line.trim_start().len()) / 2;
            let trimmed = line.trim();
            let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
            (depth, key.to_owned(), unquote(value))
        })
        .collect()
}

fn load_probes() -> Vec<Probe> {
    let mut files: Vec<PathBuf> = fs::read_dir(fixture_dir())
        .expect("routing-probes fixture directory readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("probes-")
                        && Path::new(name)
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case("lino"))
                })
        })
        .collect();
    files.sort();
    let mut probes: Vec<Probe> = Vec::new();
    for file in files {
        for (depth, key, value) in fixture_lines(&file) {
            match (depth, key.as_str()) {
                (0, _) => {}
                (1, "probe") => probes.push(Probe {
                    id: value,
                    fields: BTreeMap::new(),
                }),
                (2, _) => {
                    probes
                        .last_mut()
                        .unwrap_or_else(|| panic!("{}: field before a probe", file.display()))
                        .fields
                        .insert(key, value);
                }
                _ => panic!("{}: unexpected line `{key} {value}`", file.display()),
            }
        }
    }
    probes
}

fn load_budget() -> Budget {
    let mut budget = Budget::default();
    let mut last_lane: Option<(bool, String)> = None;
    for (depth, key, value) in fixture_lines(&fixture_dir().join("budget.lino")) {
        match (depth, key.as_str()) {
            (0, _) => {}
            (1, "extra_lane") => {
                budget.extra_lanes.insert(value.clone(), String::new());
                last_lane = Some((true, value));
            }
            (1, "exempt_lane") => {
                budget.exempt_lanes.insert(value.clone(), String::new());
                last_lane = Some((false, value));
            }
            (1, _) => {
                let number = value
                    .parse()
                    .unwrap_or_else(|error| panic!("budget `{key}` is a number: {error}"));
                budget.numbers.insert(key, number);
            }
            (2, "reason") => {
                let (extra, lane) = last_lane
                    .clone()
                    .expect("a reason follows an extra_lane or exempt_lane");
                let lanes = if extra {
                    &mut budget.extra_lanes
                } else {
                    &mut budget.exempt_lanes
                };
                lanes.insert(lane, value);
            }
            _ => panic!("budget.lino: unexpected line `{key} {value}`"),
        }
    }
    budget
}

/// The shared lanes: handler-precedence rows plus intent-routing slugs.
fn seed_lanes() -> BTreeSet<String> {
    let seed = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/seed");
    let read = |name: &str| {
        fs::read_to_string(seed.join(name))
            .unwrap_or_else(|error| panic!("data/seed/{name} readable: {error}"))
    };
    let handlers = read("handler-precedence.lino");
    let routing = read("intent-routing.lino");
    let handler_rows = handlers
        .lines()
        .filter_map(|line| line.strip_prefix("  handler "))
        .filter(|rest| !rest.starts_with(' '));
    let slugs = routing
        .lines()
        .filter_map(|line| line.strip_prefix("    slug "))
        .filter(|rest| !rest.starts_with(' '));
    handler_rows
        .chain(slugs)
        .map(|lane| lane.trim().to_owned())
        .collect()
}

#[test]
fn the_corpus_is_well_formed_and_large_enough() {
    let probes = load_probes();
    let budget = load_budget();
    assert!(
        probes.len() >= budget.number("minimum_probes"),
        "{} probes < minimum_probes",
        probes.len()
    );
    let lanes = seed_lanes();
    let mut prompts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for probe in &probes {
        for field in ["lane", "language", "intent", "prompt"] {
            assert!(!probe.field(field).is_empty(), "{} lacks {field}", probe.id);
        }
        assert!(
            LANGUAGES.contains(&probe.field("language")),
            "{}: language {}",
            probe.id,
            probe.field("language")
        );
        let lane = probe.field("lane");
        assert!(
            lanes.contains(lane) || budget.extra_lanes.contains_key(lane),
            "{}: unknown lane {lane}",
            probe.id
        );
        assert!(
            prompts.insert(probe.field("prompt").to_owned()),
            "{}: duplicate prompt",
            probe.id
        );
        assert!(ids.insert(probe.id.clone()), "duplicate id {}", probe.id);
    }
    let js_listed = probes
        .iter()
        .filter(|probe| probe.has("js_misroute"))
        .count();
    assert_eq!(
        js_listed,
        budget.number("js_misroute_ceiling"),
        "js_misroute_ceiling must equal the listed browser misroutes"
    );
    for probe in &probes {
        assert_ne!(
            probe.field("rust_misroute"),
            "unverified",
            "{}: a native misroute is the measured intent, never `unverified`",
            probe.id
        );
        assert!(
            !probe.has("rust_misroute") || probe.field("rust_misroute") != probe.native_intent(),
            "{}: a misroute equal to the intent is no misroute",
            probe.id
        );
    }
    let rust_listed = probes
        .iter()
        .filter(|probe| probe.has("rust_misroute"))
        .count();
    assert_eq!(
        rust_listed,
        budget.number("rust_misroute_ceiling"),
        "rust_misroute_ceiling must equal the listed native misroutes (it may only fall)"
    );
}

#[test]
fn every_lane_meets_the_coverage_budget_or_is_exempt_with_a_reason() {
    let probes = load_probes();
    let budget = load_budget();
    let mut lanes = seed_lanes();
    lanes.extend(budget.extra_lanes.keys().cloned());
    let minimum = budget.number("minimum_probes_per_lane");
    let mut short = Vec::new();
    for lane in &lanes {
        let count = probes
            .iter()
            .filter(|probe| {
                probe.field("lane") == lane
                    || probe.field("intent") == lane
                    || probe.field("rust_intent") == lane
            })
            .count();
        if let Some(reason) = budget.exempt_lanes.get(lane) {
            assert!(!reason.is_empty(), "exempt lane {lane} needs a reason");
            assert_eq!(
                count, 0,
                "lane {lane} is covered by {count} probes; drop its stale exemption"
            );
        } else if count < minimum {
            short.push(format!("{lane} ({count})"));
        }
    }
    for lane in budget.exempt_lanes.keys() {
        assert!(lanes.contains(lane), "exemption names unknown lane {lane}");
    }
    assert!(short.is_empty(), "lanes under {minimum} probes: {short:?}");
}

#[test]
fn the_native_solver_answers_every_probe_as_measured() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let mut moved = Vec::new();
    for probe in load_probes() {
        let answer = solver.solve(probe.field("prompt"));
        let pinned = if probe.has("rust_misroute") {
            probe.field("rust_misroute")
        } else {
            probe.native_intent()
        };
        if answer.intent == pinned {
            continue;
        }
        let correction = if answer.intent == probe.native_intent() {
            "drop its rust_misroute line and lower rust_misroute_ceiling".to_owned()
        } else {
            format!("set `rust_misroute {}`", answer.intent)
        };
        moved.push(format!(
            "{} [{}] {:?} answers {} (fixture pins {pinned}): {correction}",
            probe.id,
            probe.field("lane"),
            probe.field("prompt"),
            answer.intent,
        ));
    }
    assert!(
        moved.is_empty(),
        "{} probes answer other than the fixture pins:\n{}",
        moved.len(),
        moved.join("\n")
    );
}
