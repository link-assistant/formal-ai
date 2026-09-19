//! Conversation-container invocations: create, exec, and the name scheme
//! that makes a conversation reattachable across restarts (#937).

use super::*;

/// Stable Docker name for a conversation. The readable prefix aids operators;
/// the digest prevents punctuation, length and collision problems.
#[must_use]
pub fn conversation_container_name(conversation_id: &str) -> String {
    let digest = crate::source_fetch::sha256_hex(conversation_id.as_bytes());
    format!("formal-ai-conversation-{}", &digest[..24])
}

/// Construct the detached-container creation call without executing it.
#[must_use]
pub fn conversation_create_invocation(
    conversation_id: &str,
    image: &str,
    network: NetworkPolicy,
) -> ProcessInvocation {
    let mut arguments = vec![
        String::from("run"),
        String::from("--detach"),
        String::from("--name"),
        conversation_container_name(conversation_id),
    ];
    arguments.extend(network.container_flags());
    arguments.extend([
        image.to_owned(),
        String::from("sh"),
        String::from("-lc"),
        String::from(CONVERSATION_IDLE),
    ]);
    ProcessInvocation {
        program: String::from("docker"),
        arguments,
    }
}

/// Construct the exact execution call for an already-running conversation.
/// User program and arguments remain positional data after the fixed shell
/// program.
#[must_use]
pub fn conversation_exec_invocation(
    conversation_id: &str,
    program: &str,
    argv: &[&str],
) -> ProcessInvocation {
    let mut arguments = vec![
        String::from("exec"),
        conversation_container_name(conversation_id),
        String::from("sh"),
        String::from("-lc"),
        String::from("cd /tmp/formal-ai && exec \"$@\""),
        String::from("formal-ai"),
        program.to_owned(),
    ];
    arguments.extend(argv.iter().map(|argument| (*argument).to_owned()));
    ProcessInvocation {
        program: String::from("docker"),
        arguments,
    }
}

pub(super) fn ensure_conversation_container(
    conversation_id: &str,
    image: &str,
    network: NetworkPolicy,
) -> Result<(), BoxError> {
    if image.trim().is_empty() {
        return Err(BoxError::InvalidConfiguration {
            detail: String::from("a conversation container requires an image"),
        });
    }
    let name = conversation_container_name(conversation_id);
    let inspected = Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", &name])
        .output()
        .map_err(|error| BoxError::NoDaemon {
            detail: error.to_string(),
        })?;
    if inspected.status.success() {
        if String::from_utf8_lossy(&inspected.stdout).trim() == "true" {
            return Ok(());
        }
        let started = Command::new("docker")
            .args(["start", &name])
            .output()
            .map_err(|error| BoxError::Observed {
                detail: error.to_string(),
            })?;
        if started.status.success() {
            return Ok(());
        }
        return Err(BoxError::Observed {
            detail: String::from_utf8_lossy(&started.stderr).into_owned(),
        });
    }

    let invocation = conversation_create_invocation(conversation_id, image, network);
    let created = Command::new(&invocation.program)
        .args(&invocation.arguments)
        .output()
        .map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
    if created.status.success() {
        Ok(())
    } else {
        Err(BoxError::Observed {
            detail: String::from_utf8_lossy(&created.stderr).into_owned(),
        })
    }
}
