//! Command-line parsing and execution for the release authorship metric.
use super::*;

#[derive(Debug)]
struct Options {
    repo: PathBuf,
    since: Option<String>,
    until: String,
    ledger: PathBuf,
    record_release: Option<String>,
    /// Base revision to compare against; see [`ratchet_check`].
    check_ratchet: Option<String>,
    /// Restate every earlier-epoch row under the current definition.
    replay_epoch: bool,
    trailing_window: usize,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut repo = PathBuf::from(".");
    let mut since = None;
    let mut until = "HEAD".to_owned();
    let mut ledger = PathBuf::from(DEFAULT_LEDGER);
    let mut record_release = None;
    let mut check_ratchet = None;
    let mut replay_epoch = false;
    let mut trailing_window = DEFAULT_TRAILING_WINDOW;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let value = |args: &mut dyn Iterator<Item = String>| {
            args.next().ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--repo" => repo = PathBuf::from(value(&mut args)?),
            "--since" => since = Some(value(&mut args)?),
            "--until" => until = value(&mut args)?,
            "--ledger" => ledger = PathBuf::from(value(&mut args)?),
            "--record-release" => record_release = Some(value(&mut args)?),
            "--check-ratchet" => check_ratchet = Some(value(&mut args)?),
            "--replay-epoch" => replay_epoch = true,
            "--trailing-window" => {
                trailing_window = value(&mut args)?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --trailing-window: {error}"))?;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: self-hosting-metric.rs --since <tag> [--until <rev>] \
                     [--record-release <tag>] [--check-ratchet <base-rev>] [--replay-epoch] \
                     [--ledger <path>] [--repo <path>]"
                        .to_owned(),
                );
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if since.is_none() && check_ratchet.is_none() && !replay_epoch {
        return Err("--since <tag> is required".to_owned());
    }
    let ledger = if ledger.is_absolute() {
        ledger
    } else {
        repo.join(ledger)
    };
    Ok(Options {
        repo,
        since,
        until,
        ledger,
        record_release,
        check_ratchet,
        replay_epoch,
        trailing_window,
    })
}

pub(super) fn run() -> Result<(), String> {
    let options = parse_options(env::args().skip(1))?;
    if options.replay_epoch {
        let appended = replay_epoch(&options.repo, &options.ledger, options.trailing_window)?;
        println!(
            "restated {} release(s) under metric version {METRIC_VERSION}",
            appended.len()
        );
        return Ok(());
    }
    if let Some(base) = &options.check_ratchet {
        if let Some(regression) = ratchet_check(
            &options.repo,
            &options.ledger,
            base,
            &options.until,
            options.trailing_window,
        )? {
            return Err(regression);
        }
        println!("self-hosting ratchet holds against {base}");
        return Ok(());
    }
    let since = options
        .since
        .ok_or_else(|| "--since <tag> is required".to_owned())?;
    let measurement = if let Some(tag) = options.record_release {
        // Issue #1085 (D3.5): recording is unconditional. The self-development
        // floor is reported by `check-self-development-release.rs` from its own
        // workflow, not enforced here.
        let row = record_release_with_policy(
            &options.repo,
            &options.ledger,
            &tag,
            &since,
            &options.until,
            options.trailing_window,
            // The pull-request gate (`--check-ratchet`) is where a fall is still
            // actionable; on `main` the commits are immutable. See `RatchetPolicy`.
            RatchetPolicy::Report,
        )?;
        Measurement {
            self_authored_lines: row.self_authored_lines,
            changed_lines: row.changed_lines,
            self_authored_commits: row.self_authored_commits,
            commits: row.commits,
            percentage_basis_points: row.percentage_basis_points,
        }
    } else {
        measure(&options.repo, &since, &options.until)?
    };
    println!(
        "{} ({}/{} changed lines; {}/{} commits)",
        format_percentage(measurement.percentage_basis_points),
        measurement.self_authored_lines,
        measurement.changed_lines,
        measurement.self_authored_commits,
        measurement.commits,
    );
    Ok(())
}
