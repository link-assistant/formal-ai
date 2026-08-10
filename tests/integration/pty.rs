use std::process::Command;

#[derive(Clone, Copy)]
enum ScriptDialect {
    Bsd,
    UtilLinux,
}

pub(super) fn command(program: &str, args: &[&str]) -> Command {
    #[cfg(target_os = "macos")]
    let dialect = ScriptDialect::Bsd;
    #[cfg(not(target_os = "macos"))]
    let dialect = ScriptDialect::UtilLinux;

    command_for(dialect, program, args)
}

fn command_for(dialect: ScriptDialect, program: &str, args: &[&str]) -> Command {
    let mut command = Command::new("script");
    match dialect {
        // BSD script takes the transcript path followed by the command argv.
        ScriptDialect::Bsd => {
            command.args(["-q", "/dev/null", program]).args(args);
        }
        // util-linux script takes one shell command after -c and writes the
        // transcript to its final argument.
        ScriptDialect::UtilLinux => {
            let command_line = std::iter::once(program)
                .chain(args.iter().copied())
                .map(shell_quote)
                .collect::<Vec<_>>()
                .join(" ");
            command.args(["-qfec", &command_line, "/dev/null"]);
        }
    }
    command
}

fn shell_quote(argument: &str) -> String {
    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}

#[test]
fn bsd_script_receives_a_transcript_then_a_command_argv() {
    let command = command_for(
        ScriptDialect::Bsd,
        "/tmp/formal ai",
        &["with", "--no-start-server", "opencode"],
    );
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        args,
        [
            "-q",
            "/dev/null",
            "/tmp/formal ai",
            "with",
            "--no-start-server",
            "opencode"
        ]
    );
}

#[test]
fn util_linux_script_receives_a_quoted_shell_command() {
    let command = command_for(
        ScriptDialect::UtilLinux,
        "/tmp/formal ai",
        &["with", "it's", "opencode"],
    );
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        args,
        [
            "-qfec",
            "'/tmp/formal ai' 'with' 'it'\"'\"'s' 'opencode'",
            "/dev/null"
        ]
    );
}
