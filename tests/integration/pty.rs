use std::io::{self, BufRead as _, BufReader, Read as _, Write as _};
use std::process::{Command, Output, Stdio};

#[derive(Clone, Copy)]
enum ScriptDialect {
    Bsd,
    UtilLinux,
}

pub fn command(program: &str, args: &[&str]) -> Command {
    #[cfg(target_os = "macos")]
    let dialect = ScriptDialect::Bsd;
    #[cfg(not(target_os = "macos"))]
    let dialect = ScriptDialect::UtilLinux;

    command_for(dialect, program, args)
}

pub fn interact_after_ready(
    mut command: Command,
    ready: &[u8],
    input: &[u8],
) -> io::Result<Output> {
    if ready.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "PTY readiness marker cannot be empty",
        ));
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("PTY stderr was not piped"))?;
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        BufReader::new(stderr).read_to_end(&mut bytes)?;
        Ok::<_, io::Error>(bytes)
    });
    let child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("PTY stdout was not piped"))?;
    let mut stdout_reader = BufReader::new(child_stdout);
    let mut stdout = Vec::new();
    while !stdout.windows(ready.len()).any(|window| window == ready) {
        if stdout_reader.read_until(b'\n', &mut stdout)? == 0 {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stderr_reader.join();
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "PTY closed before its readiness marker",
            ));
        }
    }

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("PTY stdin was not piped"))?;
    stdin.write_all(input)?;
    stdin.flush()?;
    drop(stdin);

    stdout_reader.read_to_end(&mut stdout)?;
    let status = child.wait()?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| io::Error::other("PTY stderr reader panicked"))??;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
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

#[test]
fn interaction_sends_input_after_the_readiness_marker() {
    let mut command = Command::new("sh");
    command.args([
        "-c",
        "printf 'TUI_READY\\r\\n'; IFS= read -r input; [ \"$input\" = hi ]",
    ]);

    let output = interact_after_ready(command, b"TUI_READY", b"hi\n")
        .expect("complete readiness-gated interaction");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"TUI_READY\r\n");
    assert!(output.stderr.is_empty());
}
