//! Delimiting of the argv the caller wrote after the wrapped tool name.
//!
//! Kept apart from the rendering of a client invocation because this step runs
//! before any parsing: it decides where formal-ai's own flags stop and the
//! client's begin.

use std::ffi::OsString;

/// Insert the `--` separator right after the wrapped tool name.
///
/// Without it the argument parser keeps matching its own flags after the tool
/// positional, so a caller flag that happens to share a name with one of ours
/// — `--verbose` is the whole formal-ai command's global flag — is consumed by
/// formal-ai instead of reaching the client. `command` is the full parser tree,
/// so which flags take a value is read from the parser itself rather than
/// restated here.
///
/// An argv that already contains `--` is returned untouched.
#[must_use]
pub fn delimit_tool_args(argv: Vec<OsString>, command: &clap::Command) -> Vec<OsString> {
    // `Command::build` is what fills in each argument's value count; without it
    // every flag would look like a value-less switch and the token after a
    // `--base-url http://…` pair would be mistaken for the tool name.
    let mut root = command.clone();
    root.build();
    // Ancestors stay in scope because clap propagates global arguments into
    // subcommands only while parsing, not while building.
    let mut scopes = vec![&root];
    let mut wraps_tool = root.get_name() == "with-formal-ai";
    let mut index = 1;
    while index < argv.len() {
        let Some(token) = argv[index].to_str() else {
            return argv;
        };
        if token == "--" {
            return argv;
        }
        if let Some(consumed) = flag_token_length(token, &scopes) {
            index += consumed;
            continue;
        }
        if wraps_tool {
            // A caller who already wrote the delimiter themselves gets it back
            // untouched: a second one would reach the client verbatim.
            if argv.get(index + 1).is_some_and(|next| next == "--") {
                return argv;
            }
            let mut delimited = argv;
            delimited.insert(index + 1, OsString::from("--"));
            return delimited;
        }
        let Some(subcommand) = scopes.last().and_then(|scope| scope.find_subcommand(token)) else {
            return argv;
        };
        wraps_tool = subcommand.get_name() == "with";
        scopes.push(subcommand);
        index += 1;
    }
    argv
}

/// How many argv entries `token` occupies when it is a flag, or `None` when it
/// is a positional.
///
/// `scopes` is the chain from the root command to the one currently being
/// parsed; only the innermost scope contributes its local arguments, the outer
/// ones contribute their global arguments the way clap propagates them.
fn flag_token_length(token: &str, scopes: &[&clap::Command]) -> Option<usize> {
    if let Some(long) = token.strip_prefix("--") {
        if long.is_empty() {
            return None;
        }
        if long.contains('=') {
            return Some(1);
        }
        let takes_value = value_args(scopes).any(|argument| {
            argument
                .get_long_and_visible_aliases()
                .is_some_and(|names| names.contains(&long))
        });
        return Some(1 + usize::from(takes_value));
    }
    let shorts = token
        .strip_prefix('-')
        .filter(|shorts| !shorts.is_empty())?;
    for (position, short) in shorts.char_indices() {
        let takes_value = value_args(scopes).any(|argument| {
            argument
                .get_short_and_visible_aliases()
                .is_some_and(|names| names.contains(&short))
        });
        if takes_value {
            // A value attached to the cluster (`-mvalue`) keeps the flag to one
            // entry; a detached one takes the next entry too.
            let attached = position + short.len_utf8() < shorts.len();
            return Some(if attached { 1 } else { 2 });
        }
    }
    Some(1)
}

/// Every value-taking option visible in the innermost scope.
fn value_args<'a>(scopes: &'a [&'a clap::Command]) -> impl Iterator<Item = &'a clap::Arg> {
    let depth = scopes.len();
    scopes
        .iter()
        .enumerate()
        .flat_map(move |(level, scope)| {
            scope
                .get_arguments()
                .filter(move |argument| level + 1 == depth || argument.is_global_set())
        })
        .filter(|argument| {
            argument
                .get_num_args()
                .is_some_and(|count| count.takes_values())
        })
}
