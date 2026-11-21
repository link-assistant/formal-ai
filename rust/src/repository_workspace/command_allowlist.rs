use super::{ALLOWLIST_LINO, RECORD_ALLOWED};
use crate::seed::parser::parse_lino;

/// One row of `data/seed/repository-command-allowlist.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedCommand {
    /// The row's stable id.
    pub id: String,
    /// The program a shell would resolve.
    pub program: String,
    /// The subcommand, when the row names one.
    pub subcommand: Option<String>,
    /// The argument shape, with `{placeholders}`.
    pub arguments: Vec<String>,
    /// Whether the command changes the tree.
    pub mutating: bool,
    /// What must hold before it runs.
    pub precondition: Option<String>,
    /// What must be observed after it.
    pub postcondition: Option<String>,
}

/// The seed allowlist, in file order. Default-deny: a program with no row is
/// refused, and so is a listed program with an unlisted subcommand.
#[must_use]
pub fn command_allowlist() -> Vec<AllowedCommand> {
    parse_lino(ALLOWLIST_LINO)
        .children
        .iter()
        .filter(|node| node.find_child_value("record_type") == RECORD_ALLOWED)
        .map(|node| AllowedCommand {
            id: node.find_child_value("id").to_owned(),
            program: node.find_child_value("program").to_owned(),
            subcommand: optional(node.find_child_value("subcommand")),
            arguments: crate::prerequisite::probe::split_tuple(node.find_child_value("arguments")),
            mutating: node.find_child_value("mutating") == "true",
            precondition: optional(node.find_child_value("precondition")),
            postcondition: optional(node.find_child_value("postcondition")),
        })
        .collect()
}

/// The bound used when no row declares one.
pub const DEFAULT_DEADLINE_SECONDS: u64 = 300;

/// The bound a permitted command shape is measured against, in seconds.
///
/// Declared per shape rather than globally, because "it did not finish" means
/// something different for a one-line script and for a whole suite, and one
/// number for both would make the shorter one an allowance or the longer one a
/// guillotine.
#[must_use]
pub fn deadline_seconds(program: &str, argv: &[&str]) -> u64 {
    let tail = subcommand_of(program, argv);
    let subcommand = tail.first().copied().unwrap_or_default();
    parse_lino(ALLOWLIST_LINO)
        .children
        .iter()
        .find(|node| {
            node.find_child_value("record_type") == RECORD_ALLOWED
                && node.find_child_value("program") == program
                && node.find_child_value("subcommand") == subcommand
        })
        .and_then(|node| node.find_child_value("deadline_seconds").parse().ok())
        .unwrap_or(DEFAULT_DEADLINE_SECONDS)
}

/// A field that is present only when it is non-empty.
fn optional(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

/// `git`'s global options, which precede the subcommand and are not one.
///
/// Scoped to `git` deliberately. `-c` is a global option there and the whole
/// subcommand elsewhere — `python3 -c` is the inline-script form — so stripping
/// it unconditionally would leave an interpreter with no subcommand at all, and
/// a default-deny table would then refuse a shape it names.
const GIT_GLOBAL_FLAGS: &[&str] = &["-C", "-c"];

/// The program whose leading options are global rather than subcommands.
const GLOBAL_FLAG_PROGRAM: &str = "git";

/// Strip the leading global options so the subcommand is found where a row
/// names it, and never mistaken for one.
fn subcommand_of<'a>(program: &str, argv: &'a [&'a str]) -> &'a [&'a str] {
    if program != GLOBAL_FLAG_PROGRAM {
        return argv;
    }
    let mut index = 0;
    while index + 1 < argv.len() && GIT_GLOBAL_FLAGS.contains(&argv[index]) {
        index += 2;
    }
    &argv[index..]
}

/// Whether `program` with `argv` is permitted by the seed allowlist.
///
/// Default-deny in three places at once: a program with no row is refused, a
/// listed program with an unlisted subcommand is refused, and a listed
/// subcommand whose arguments do not fit the row's shape is refused. That is
/// what makes a sentence unrunnable — its first word names no program, so it
/// never reaches a shell to fail there.
#[must_use]
pub fn allows(program: &str, argv: &[&str]) -> bool {
    let tail = subcommand_of(program, argv);
    let Some((subcommand, arguments)) = tail.split_first() else {
        return false;
    };
    command_allowlist().into_iter().any(|row| {
        row.program == program
            && row.subcommand.as_deref() == Some(*subcommand)
            && shape_matches(&row.arguments, arguments)
    })
}

/// Whether `arguments` fit `shape`. A `{placeholder}` matches exactly one
/// argument; `{rest}` as the last token matches any remainder; a literal must
/// match exactly.
fn shape_matches(shape: &[String], arguments: &[&str]) -> bool {
    let mut index = 0;
    for (position, token) in shape.iter().enumerate() {
        if token == "{rest}" {
            return position + 1 == shape.len();
        }
        let Some(argument) = arguments.get(index) else {
            return false;
        };
        if !token.starts_with('{') && token != argument {
            return false;
        }
        index += 1;
    }
    index == arguments.len()
}
