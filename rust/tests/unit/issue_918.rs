//! Regression coverage for issue #918's minimal-core and seed-metadata audit.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The CI gate itself, compiled into the suite.
///
/// The census below and `rust-script scripts/check-minimal-core-boundary.rs`
/// must agree on which files are handler debt. Walking the directory twice made
/// that a coincidence; reading it through the gate's own `source_files` makes it
/// a fact -- issue #991 added an exclusion for generated `mod` lists, and one
/// edit taught both.
#[path = "../../../scripts/check-minimal-core-boundary.rs"]
mod check_minimal_core_boundary;

/// Every handler source the boundary gate counts, with its line count, relative
/// to the repository root.
fn handler_source_lines() -> BTreeMap<String, usize> {
    check_minimal_core_boundary::source_files(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the repository root sits one level above the crate"),
    )
    .expect("the boundary gate should enumerate the handler sources")
}

/// Every handler source the boundary gate counts, relative to the repository
/// root.
fn handler_sources() -> BTreeSet<String> {
    handler_source_lines().into_keys().collect()
}

#[derive(Default)]
struct LedgerEntry {
    path: String,
    disposition: String,
    core_component: String,
    reason: String,
}

fn ledger_entries(ledger: &str) -> Vec<LedgerEntry> {
    let mut entries = Vec::new();
    let mut current: Option<LedgerEntry> = None;
    for line in ledger.lines() {
        if let Some(path) = line.strip_prefix("  source ") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(LedgerEntry {
                path: path.to_owned(),
                ..LedgerEntry::default()
            });
        } else if let Some(entry) = current.as_mut() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("disposition ") {
                value.clone_into(&mut entry.disposition);
            } else if let Some(value) = trimmed.strip_prefix("core_component ") {
                value.clone_into(&mut entry.core_component);
            } else if let Some(value) = trimmed.strip_prefix("reason ") {
                value.trim_matches('"').clone_into(&mut entry.reason);
            }
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    entries
}

fn lino_files_below(directory: &Path) -> Vec<PathBuf> {
    fn visit(directory: &Path, paths: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("data directory") {
            let path = entry.expect("data entry").path();
            if path.is_dir() {
                visit(&path, paths);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "lino")
            {
                paths.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    visit(directory, &mut paths);
    paths.sort();
    paths
}

fn required_metadata(schema: &str) -> Vec<String> {
    schema
        .lines()
        .filter_map(|line| line.strip_prefix("  required_field "))
        .map(str::to_owned)
        .collect()
}

fn meaning_records(source: &str, text: &str) -> Vec<(String, String, BTreeSet<String>)> {
    if text.lines().find(|line| !line.trim().is_empty()) != Some("meanings") {
        return Vec::new();
    }
    let mut records = Vec::new();
    let mut current: Option<(String, String, BTreeSet<String>)> = None;
    for line in text.lines().skip(1) {
        let indentation = line.bytes().take_while(|byte| *byte == b' ').count();
        if indentation == 2 {
            if let Some(record) = current.take() {
                records.push(record);
            }
            current = Some((
                source.to_owned(),
                line.split_whitespace()
                    .next()
                    .expect("meaning name")
                    .to_owned(),
                BTreeSet::new(),
            ));
        } else if indentation == 4 {
            let trimmed = line.trim();
            if let (Some(record), Some(value_offset)) =
                (current.as_mut(), trimmed.find(char::is_whitespace))
            {
                let field = &trimmed[..value_offset];
                let raw_value = trimmed[value_offset..].trim();
                let value = raw_value
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                    .unwrap_or(raw_value);
                if !value.trim().is_empty() {
                    record.2.insert(field.to_owned());
                }
            }
        }
    }
    if let Some(record) = current {
        records.push(record);
    }
    records
}

fn committed_gaps(root: &Path) -> BTreeMap<(String, String), String> {
    let mut paths = fs::read_dir(root.join("data/meta"))
        .expect("metadata directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().is_some_and(|name| {
                let name = name.to_string_lossy();
                name.starts_with("seed-metadata-gaps-") && name.ends_with(".lino")
            })
        })
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(
        paths.len(),
        16,
        "the metadata gap audit uses 16 stable shards"
    );

    let mut gaps = BTreeMap::new();
    for path in paths {
        let text = fs::read_to_string(&path).expect("metadata gap shard");
        assert!(
            text.lines().count() <= 1_500,
            "{} exceeds the data-file line limit",
            path.display()
        );
        let mut gap_id = None;
        let mut record = None;
        let mut source = None;
        for line in text.lines() {
            if let Some(value) = line.strip_prefix("  gap ") {
                gap_id = Some(value.to_owned());
                record = None;
                source = None;
            } else if let Some(value) = line.strip_prefix("    source ") {
                source = Some(value.trim_matches('"').to_owned());
            } else if let Some(value) = line.strip_prefix("    record ") {
                record = Some(value.trim_matches('"').to_owned());
            } else if let Some(value) = line.strip_prefix("    missing ") {
                let id = gap_id.take().expect("gap id");
                assert!(
                    id.starts_with("seed_metadata_gap_") && id.len() == 34,
                    "gap identifier should be a stable 64-bit hash: {id}"
                );
                let key = (
                    source.take().expect("gap source"),
                    record.take().expect("gap record"),
                );
                assert!(
                    gaps.insert(key, value.trim_matches('"').to_owned())
                        .is_none(),
                    "each seed record has at most one gap entry"
                );
            }
        }
    }
    gaps
}

#[test]
fn minimal_core_ledger_covers_every_recursive_handler_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let actual = handler_sources();
    let ledger = fs::read_to_string(root.join("data/meta/core-boundary-ledger.lino"))
        .expect("issue #918 must provide the source-file core-boundary ledger");
    let entries = ledger_entries(&ledger);
    let active = entries
        .iter()
        .filter(|entry| entry.disposition != "delete")
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(active, actual);
    // 46 until issue #1085 moved `github_repository_traffic.rs` into
    // `data/seed/handler-rules.lino`; the ledger and the tree dropped together.
    // 45 until issue #1138 B9, plan 09 leaf 1: the gate scanned
    // `src/solver_handlers` only, so `solver_handler_how.rs`,
    // `solver_handler_how_synthesis.rs`, `solver_handler_units.rs` and
    // `solver_handler_oracle.rs` — four handler files one directory up — were
    // neither counted nor ledgered, and a migration could have lowered the
    // ratchet by moving a file out of the scanned directory. The rise to 49 is a
    // corrected undercount recorded in the ledger's `note`, not new debt.
    // Plan 08 then added one generic interpreter of seed-declared verifiable
    // tasks. It is compiled core machinery rather than another domain handler.
    // The URL parsing split then exposed another generic interpreter. Its
    // structural parser reads language evidence from seed roles, so recursive
    // source count rises while migration debt continues to fall.
    // The plan 16 L1 re-measure then ledgered the calendar-create split and the
    // previously unledgered policy_gates.rs, so the honest census is 53 files.
    assert_eq!(actual.len(), 53);
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.disposition == "migrate")
            .count(),
        51
    );
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.disposition == "promote")
            .count(),
        2,
        "both generic seed-driven interpreters are promoted"
    );
    for entry in entries
        .iter()
        .filter(|entry| entry.disposition == "promote")
    {
        assert!(
            !entry.core_component.is_empty(),
            "{} promotion must name its core component",
            entry.path
        );
        assert!(
            !entry.reason.is_empty(),
            "{} promotion must explain the boundary decision",
            entry.path
        );
    }
    for disposition in entries.iter().map(|entry| entry.disposition.as_str()) {
        assert!(matches!(disposition, "migrate" | "promote" | "delete"));
    }
    // The ledger's ceiling is the measurement itself: the gate fails when it sits
    // above *or* below the real count, so migration debt can only ratchet down.
    // Restating the number here would freeze a value that every handler edit moves
    // into a second shared file -- the conflict shape issue #991 removes -- so the
    // expected total is summed from the same sources the gate measures.
    let lines = handler_source_lines();
    let outside_core_lines: usize = entries
        .iter()
        .filter(|entry| entry.disposition == "migrate")
        .map(|entry| lines[&entry.path])
        .sum();
    assert!(ledger.contains(&format!("outside_core_lines_max {outside_core_lines}")));
}

#[test]
fn generic_interpreter_components_are_registered_outside_the_census() {
    // Issue #1138 B9, plan 09 leaf 17: the migration families need compiled
    // homes that are not themselves handler debt. A `component` block in the
    // ledger names one such file outside the census, the boundary category it
    // was promoted under, and the reason it passes the promotion test. The
    // test reads the ledger through the gate's own parser so the gate and the
    // suite cannot disagree about what a component is.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let ledger_text = fs::read_to_string(root.join("data/meta/core-boundary-ledger.lino"))
        .expect("issue #918 must provide the source-file core-boundary ledger");
    let ledger = check_minimal_core_boundary::parse_ledger(&ledger_text)
        .expect("the boundary ledger parses, components included");
    let census = handler_sources();
    assert!(
        !ledger.components.is_empty(),
        "the migration families register their generic interpreters as components"
    );
    let mut names = BTreeSet::new();
    for record in &ledger.components {
        assert!(
            names.insert(record.name.as_str()),
            "component names are unique: {}",
            record.name
        );
        assert_eq!(
            record.kind, "Generic interpreter",
            "component {} is registered under the boundary document's interpreter category",
            record.name
        );
        assert!(
            !record.reason.is_empty(),
            "component {} explains the promotion decision",
            record.name
        );
        assert!(
            !census.contains(&record.file),
            "component {} file {} must sit outside the handler census",
            record.name,
            record.file
        );
        assert!(
            root.join(&record.file).is_file(),
            "component {} file {} must exist",
            record.name,
            record.file
        );
    }
    assert!(
        names.contains("retrieval_method_interpreter"),
        "the M2 retrieval family's interpreter is registered"
    );
}

#[test]
fn boundary_document_names_the_only_four_compiled_core_categories() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let boundary = fs::read_to_string(root.join("docs/design/minimal-core-boundary.md"))
        .expect("minimal-core boundary document");
    for category in [
        "Meta algorithm",
        "Link store",
        "Generic interpreters",
        "Host surfaces",
    ] {
        assert!(
            boundary.contains(category),
            "missing boundary category {category}"
        );
    }
    assert!(boundary.contains("Mixed files fail the promotion test"));
    assert!(boundary.contains("Everything else is seed or learned data"));
}

#[test]
fn coding_path_has_complete_metadata_and_every_other_gap_is_data() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let schema_text = fs::read_to_string(root.join("data/meta/seed-metadata-schema.lino"))
        .expect("seed metadata schema");
    let required = required_metadata(&schema_text);
    assert_eq!(
        required,
        ["role", "precondition", "effect", "unit", "example"]
    );
    for external_shape in ["FrameNet", "Wikidata:Data_model", "typed property value"] {
        assert!(
            schema_text.contains(external_shape),
            "schema must cite {external_shape}"
        );
    }
    let complete_sources = schema_text
        .lines()
        .filter_map(|line| line.strip_prefix("  complete_source "))
        .map(|value| value.trim_matches('"').to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(complete_sources.len(), 3);
    let registered_languages = formal_ai::language::registered_languages();
    assert!(!registered_languages.is_empty(), "language registry");

    let mut expected_gaps = BTreeMap::new();
    let mut coding_records = 0;
    let seed_root = root.join("data/seed");
    for path in lino_files_below(&seed_root) {
        let source = path
            .strip_prefix(root)
            .expect("relative seed path")
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&path).expect("seed file");
        for (source, record, fields) in meaning_records(&source, &text) {
            let missing = required
                .iter()
                .filter(|field| !fields.contains(field.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if complete_sources.contains(&source) {
                coding_records += 1;
                for language in &registered_languages {
                    assert!(
                        missing.is_empty(),
                        "{source}:{record} is missing language-neutral metadata {missing:?} for {}",
                        language.slug()
                    );
                }
            } else if !missing.is_empty() {
                expected_gaps.insert((source, record), missing.join(","));
            }
        }
    }
    // Scala and Kotlin joined the coding catalog for the hive-mind#2158
    // language matrix (issue #921), and PHP followed them under issue #1021, so
    // the complete-source floor rises with each: 27 catalog records + 15 task
    // records. The fifteenth task is `program_task_copy_stdin_to_stdout`, the
    // task issue #863 reported (see `tests/unit/issue_1021_behaviour_range.rs`);
    // the twenty-seventh catalog record is `program_language_laravel`, the
    // framework issue #723 asked for, which is a catalog row of its own rather
    // than an alias surface of PHP. Either way a new target is a new
    // complete-source record, never a new gap. Three later additions are not
    // language targets at all: `workspace_inspection_examine`,
    // `workspace_inspection_verify` and `workspace_inspection_identify` are the
    // vocabulary that lets a request to *look at* the repository route to a
    // workspace search instead of an open-web one (issue #1066). They land in
    // the same complete source as the coding tasks and carry the same five
    // fields, so they raise the floor rather than the gap count -- which is the
    // point of the floor: new agent capability arrives as described data.
    // Issue #1069 first added 29 complete workspace-search records, moving the
    // floor from 45 to 74. Its verified ladder then supplied 18 leaf-first
    // decomposition and proof facts, moving the floor from 74 to 92. Its
    // change-shaped delegation work added the three member-list kinds --
    // `coding_member_list_sequence`, `coding_member_list_set` and
    // `coding_member_list_group` -- that let a request name the collection it
    // edits in words the file never has to repeat, moving the floor from 92 to
    // 95. Issue #1131 added `coding_member_add`, the verb that says a member
    // list should grow, moving the floor from 95 to 96: the nouns alone had
    // been routing whole-file rewrites into member insertion. Issue #1133
    // added five more records from the three Hive Mind runs that produced no
    // solution -- `git_branch_cue` and `git_commit_request` (where a work
    // item's result lands, and the request to land it), `ci_workflow_request`
    // (the workflow an issue asks for beside the program) and the two
    // `file_edit_position_*` cues of an additive edit -- moving the floor
    // from 96 to 101: each was a hardcoded English sentence or a missing route
    // before it was data. None is allowed to become a metadata gap.
    // Issue #1138 moved five repository-workflow meanings out of the coding
    // task source into their own complete domain source, then added four
    // language-neutral repository-target meanings here. Recounted 2026-09-22,
    // the three complete coding sources carry 5 (config) + 28 (catalog) +
    // 68 (tasks) = 101 records: the #1138 net-floor line said 100 and shipped
    // unverified, masked by a run whose census-freshness step died before the
    // unit tests executed. The floor is 101; none of the nine records became
    // an unreviewed metadata gap.
    assert_eq!(coding_records, 101, "coding-path regression floor");
    assert_eq!(committed_gaps(root), expected_gaps);
    // The floor moves with the closure, not with the handlers: every gap added
    // under issue #1021 is a `closure-generated-*.lino` record for a token the
    // new prose pulled into the total closure -- 74 for the contribution
    // artifacts, then 16 more when the listing detector was given its Spanish
    // vocabulary (`lista`, `ficheros`, `archivos`, `aqui`, ...), then 8 more
    // for the closed-class words the language-less coding request subtracts
    // (`me`, `my`, `our`, `we`, `some`, `these`, `just`) plus the Spanish
    // `guion`, and one more for `copy_stdin_to_stdout` once the portable seed
    // bundle started naming the task issue #863 reported. A new language's
    // words arriving as generated closure records rather than as hand-written
    // gaps is the shape this floor exists to show; `request_function_word`
    // joins the count as a hand-written record because every meaning in
    // `data/seed/meanings.lino` -- all 22 of them, not just the new one --
    // carries its lexemes without the five-field metadata that file has never
    // supplied. The last twelve arrived the same way: the verified-mutating-
    // action responses issues #824 and #944 added pull their two intent tokens
    // and their ten response ids into the closure, and every one of the twelve
    // is a `closure-generated-*.lino` record -- no hand-written gap, no
    // handler without metadata. The one after them is hand-written and says so
    // honestly: `file_leading_line_constraint` in
    // `data/seed/meanings-file-write.lino` is the vocabulary for "the first
    // line must be ..." that lets an evidence-record request pin its opening
    // line (issue #1066), and that file, like `data/seed/meanings.lino`, has
    // never carried the five fields for any of its records. A gap recorded in
    // reviewed data is the outcome this audit is for; the floor above is what
    // forbids the same omission on the coding path.
    //
    // The seven after it are closure records again, and they arrived for the
    // same reason the twelve before them did. `data/seed/meanings-note-
    // composition.lino` carries both of its meanings with all five fields --
    // it is a hand-written file and it is complete -- but the closure expands
    // each of its English surfaces into a generated record, and `produce`,
    // `compose`, `draft`, `assemble`, `prepare`, `note` and `memo` had no
    // generated record before. `report`, `summary` and `brief` already did.
    // Vocabulary for a new capability entering the closure is exactly the
    // shape this number tracks: described data grows the count, an
    // undescribed handler would not be allowed to.
    //
    // The eleven after those are the same two shapes again, and they arrived
    // together under issue #1066. Ten are closure records for the honest
    // replies a decomposition gives when it cannot enumerate anything: the two
    // intent tokens `task_decomposition_single_need` and
    // `task_decomposition_unsplit_depth_bound`, and their eight response ids,
    // one per language the seed then answered in. The eleventh is hand-written
    // and says so:
    // `file_write_deferred_content` in `data/seed/meanings-file-write.lino` is
    // the vocabulary for a payload that *names* the work product ("... the
    // findings", "... 结论") rather than stating it, which is what stops a
    // request to record findings from writing the words "the findings" into
    // the caller's file. That file carries `role` and has never carried the
    // other four for any record, so the new one is a gap on exactly the terms
    // its neighbours already are.
    // The thirteen after those are one response id per decomposition intent,
    // and they arrived when Spanish stopped being answered in English (issue
    // #1066). `es` is a registered language, `data/seed/multilingual-responses-
    // decomposition.lino` carried no record in it, and `localized_response`
    // falls back to English rather than failing -- so a Spanish speaker asking
    // why nothing could be enumerated was told something true in words they had
    // not asked in. Filling the hole is described data entering the closure,
    // which is the growth this number exists to track. Leaving the fallback in
    // place would have held the count still and kept the answer wrong, which is
    // the direction this floor is here to make visible.
    // Issue #1069 first contributes 33 more reviewed closure gaps: the two new
    // repository-observation intents and ten localized response ids, plus the
    // source-search vocabulary generated from the 29 complete coding records
    // above. The exact-composition, failed multi-read and link-publish response
    // seeds then add 18 generated records while displacing the three obsolete
    // render-surface tokens, for a net 15 more gaps. They are closure data, not
    // missing metadata on the coding path. Six more arrive with the member-list
    // kinds: `set`, `collection`, `group`, `lists`, `sequence` and
    // `alternation` are the English surfaces that let a request name the
    // collection it edits, and the closure expands each into a generated
    // record. Vocabulary for a new capability entering the closure is the same
    // shape as every addition above it.
    // The last 24 are the sentences a change route says its change in, and they
    // are four intents plus one response id per registered language for each:
    // `coding_member_inserted`, `coding_member_already_present`,
    // `coding_text_replaced` and `coding_identifier_renamed`. A route that
    // reported only `{path}` told a caller which file it had touched and never
    // what it had done to it, which is not a record of a change -- the issue
    // #1028 ladder asks each leaf's proof for "at least four words that state
    // the change you made", and a status line has none of them. Naming the
    // members that went into a list, or the name that became which other name,
    // is text a person reads, so it is seed text in all five languages
    // `data/seed/languages.lino` registers, exactly as the other fifteen
    // intents in `data/seed/multilingual-responses-agentic-tools.lino` are.
    // Answering in English only would have held this number 20 lower and left
    // four of the five languages hearing about their own edit in a language
    // they had not asked in.
    // The four after them are the primacy kinds issue #1073 added: `citation`,
    // `first_hand_record`, `self_published` and `editorial_synthesis` are the
    // vocabulary a source uses to say how close it stands to what it reports,
    // and every source in `data/seed/sources-registry.lino` now names one of
    // them in its `primacy` chain instead of asserting a tier. The closure
    // expands each into a generated record, so computing trust rather than
    // assuming it arrives here as described data, on the same terms as every
    // vocabulary addition above.
    // The last forty-two arrive the same way, from issue #1075's
    // `data/seed/tool-resource-scopes.lino`: the words that let a tool say
    // *where* its effect lands rather than only what it does --
    // `remote_service`, `process_input`, `connector`, `namespaced`,
    // `substring`, `router`, `advertised`, `grounded`, `untouched` and the
    // prose around them. Every one is a `closure-generated-*.lino` record for a
    // token the new seed prose pulled into the total closure, not a
    // hand-written gap, which is the shape this floor exists to show.
    //
    // The last fifty-four arrive that way too, from issue #1085: the tokens
    // `data/seed/handler-rules.lino` and
    // `data/seed/multilingual-responses-policy.lino` introduced when eleven
    // handlers stopped being Rust -- the rule vocabulary (`agent_info`,
    // `backticks`, `cleaned`, `raw`, `literal`, `prompt`, `forms`), the
    // handler names themselves, and the response ids of their four-language
    // wording. Behaviour moving from Rust into seed data arrives as generated
    // closure records, which is what this floor is for. Four more followed when
    // those responses were given their Spanish text, which every supported
    // locale owes the others. Ten more arrived the same way with issues #1095
    // and #1099: two seed roles (`agentic_continuation_cue`,
    // `enumeration_cue`), the `agentic_continuation` handler and its
    // `continuation_cue` rule, that handler's five-language wording, and the
    // seeded notice `context_session_guessed_notice` from #1105 RC6. Every one
    // is behaviour that used to be Rust -- a hardcoded English phrase, an
    // exact-string comparison in the planner -- arriving as described data,
    // which is what this floor exists to record. Issue #1131 brought five more
    // through the total closure (`adds`, `appends`, `include`, `insert`,
    // `inserts`): the surfaces of `coding_member_add`, the verb that has to be
    // present before a member list may be grown.
    // Issue #1133 moved the total from 4,059 to 4,062: five coding-path
    // records arrived (see the coding floor above) while the Spanish and
    // "resolve" lexemes given to `implement` closed two gaps that had been
    // waiting on exactly that wording.
    // Issue #710 moved the total from 4,062 to 4,250: the generalized coding
    // structures, discoverable runtime templates, and source-backed recurrence
    // vocabulary entered the total closure instead of remaining Rust literals.
    //
    // Issue #1138 B9, plan 09 leaf 8 moved it from 4,250 to 695, and the drop is
    // the point of the leaf rather than progress against this floor. 3,555 of
    // those 4,250 rows were records in `data/seed/closure-generated-*.lino` --
    // files `scripts/close-total.py` wrote and `scripts/audit-total-closure.py`
    // then read back as definitions, so the closure metric reported zero while
    // 17,791 lines of English-only glosses no runtime loads stood in for
    // grounding. Every one of them was a metadata gap by construction: a
    // generated record carries `defined-by` and an English `lexeme` and none of
    // the five reviewed fields. Deleting the generator's output did not ground
    // anything, and the grounding work it was standing in for is now counted
    // honestly, in one place, by `data/meta/closure-audit.lino` -- 3,569 distinct
    // tokens as measured on 2026-09-16. The 695 rows that remain are the
    // hand-written gaps this audit has always been about.
    //
    // The pin then drifted twice while every Test leg that should have caught
    // it died at the census-freshness step before the unit tests ran:
    // 8c831a451 added 28 rows and e8533a522 added 13 more -- both ran the
    // audit script with --write so the shards told the truth while this
    // assert kept the old count. Recounted 2026-09-22 the shards hold 737,
    // including the `personal_facts_listing_request` role the personal-facts
    // paydown added: its five reviewed fields are pending reviewed data, so
    // it is a gap row like its siblings, not a silent omission.
    assert_eq!(expected_gaps.len(), 737);
}
