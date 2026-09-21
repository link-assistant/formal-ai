//! Shell-invocation and path helpers shared by the execution backends.

use super::{BoxError, Command, Path, PathBuf, ProcessInvocation};

pub(super) fn prefixed_invocation(
    runner: &str,
    prefix: &[String],
    program: &str,
    arguments: &[&str],
) -> ProcessInvocation {
    let mut argv = prefix.to_vec();
    argv.push(program.to_owned());
    argv.extend(arguments.iter().map(|argument| (*argument).to_owned()));
    ProcessInvocation {
        program: runner.to_owned(),
        arguments: argv,
    }
}

pub(super) fn validate_image(image: &str) -> Result<(), BoxError> {
    let valid = !image.is_empty()
        && !image.starts_with('-')
        && image.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-' | b':' | b'@')
        });
    if valid {
        Ok(())
    } else {
        Err(BoxError::InvalidConfiguration {
            detail: String::from("container image is not a safe Docker reference"),
        })
    }
}

pub(super) fn checked_workspace_path(root: &Path, relative: &Path) -> Result<PathBuf, BoxError> {
    let mut path = root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(BoxError::Observed {
                detail: String::from("input path is not workspace-relative"),
            });
        };
        path.push(part);
        if std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(BoxError::Observed {
                detail: String::from("input path crosses a symbolic link"),
            });
        }
    }
    Ok(path)
}

pub(super) fn archive_workspace(workspace: &Path) -> Result<Vec<u8>, BoxError> {
    let output = Command::new("tar")
        .args(["-czf", "-", "-C"])
        .arg(workspace)
        .arg(".")
        .output()
        .map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
    if !output.status.success() {
        return Err(BoxError::Observed {
            detail: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output.stdout)
}

pub(super) fn swebench_image(instance_id: &str) -> String {
    let safe: String = instance_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect();
    format!("swebench/sweb.eval.x86_64.{safe}:latest")
}
