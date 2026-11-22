//! Project creator claims retrieved from their registered publisher declarations.

use serde_json::Value;

use super::factual_qa::render_template;
use super::finalize_simple;
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::fact_live::{live_fact_questions, role_surfaces};
use crate::language::detect;
use crate::seed::{self, ProjectRecord};
use crate::source_fetch::{CachedSourceClient, SourceTransport};

/// Match the subject of a creator question to a project with a declaration.
fn origin_project(prompt: &str) -> Option<ProjectRecord> {
    let question = live_fact_questions(prompt)
        .into_iter()
        .find(|question| question.relation == "inventor")?;
    let registry = seed::projects_registry();
    let project =
        if role_surfaces(seed::ROLE_ASSISTANT_SELF_REFERENCE).contains(&question.subject_term) {
            registry
                .projects
                .iter()
                .find(|project| project.assistant_project)
        } else {
            registry.by_alias(&question.subject_term)
        }?;
    (!project.origin_declaration.is_empty()).then(|| project.clone())
}

pub(super) fn names_project_origin(prompt: &str) -> bool {
    origin_project(prompt).is_some()
}

fn numeric_identifier(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// Read the issue number and comment ID from a declaration in the project's repository.
fn origin_reference(project: &ProjectRecord) -> Option<(&str, &str)> {
    let (issue, comment) = project
        .origin_declaration
        .strip_prefix(&project.url)?
        .strip_prefix("/issues/")?
        .split_once("#issuecomment-")?;
    (numeric_identifier(issue) && numeric_identifier(comment)).then_some((issue, comment))
}

fn github_name(value: &str, extra: &[u8]) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || extra.contains(&byte))
}

fn github_repository(link: &str) -> bool {
    link.strip_prefix("github.com/")
        .and_then(|path| path.split_once('/'))
        .is_some_and(|(owner, repository)| {
            github_name(owner, b"-") && github_name(repository, b"-_.")
        })
}

/// Accept a creator's first-party declaration, with an optional tool repository.
fn creator_statement<'a>(
    project: &ProjectRecord,
    data: &'a Value,
) -> Option<(&'a str, String, &'a str)> {
    if !matches!(
        data.get("author_association")?.as_str()?,
        "OWNER" | "MEMBER" | "COLLABORATOR" | "CONTRIBUTOR"
    ) {
        return None;
    }
    let author = data.pointer("/user/login")?.as_str()?;
    if !github_name(author, b"-") {
        return None;
    }
    let (subject, remainder) = data
        .get("body")?
        .as_str()?
        .trim()
        .strip_suffix('.')?
        .split_once(" was created by github.com/")?;
    if normalize_prompt(subject) != normalize_prompt(&project.display_name)
        && !project.matches_alias(subject)
    {
        return None;
    }
    let (owner, tool) = remainder.split_once(" using ").unwrap_or((remainder, ""));
    if !owner.eq_ignore_ascii_case(author) || (!tool.is_empty() && !github_repository(tool)) {
        return None;
    }
    Some((subject, format!("github.com/{author}"), tool))
}

pub(super) fn try_project_origin<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let project = origin_project(prompt)?;
    let (issue, comment) = origin_reference(&project)?;
    let repository = project.repo_slug();
    let issue_url = format!("https://api.github.com/repos/{repository}/issues/{issue}");
    let api = format!("https://api.github.com/repos/{repository}/issues/comments/{comment}");
    log.append("project_origin:request", api.as_str());
    let capture = match client.fetch(&api) {
        Ok(capture) => capture,
        Err(error) => {
            log.append("project_origin:unavailable", error.to_string());
            return None;
        }
    };
    let data: Value = serde_json::from_slice(capture.bytes()).ok()?;
    let expected_identifier = comment.parse::<u64>().ok()?;
    if data.get("id")?.as_u64()? != expected_identifier
        || data.get("html_url")?.as_str()? != project.origin_declaration
        || data.get("issue_url")?.as_str()? != issue_url
    {
        log.append("project_origin:rejected", "publisher_identity");
        return None;
    }
    let Some((subject, author, tool)) = creator_statement(&project, &data) else {
        log.append("project_origin:rejected", "creator_statement");
        return None;
    };
    let template = if tool.is_empty() {
        "project-origin-author"
    } else {
        "project-origin-author-and-tool"
    };
    let body = render_template(
        template,
        detect(prompt).slug(),
        &[("subject", subject), ("author", &author), ("tool", tool)],
    );
    if body.is_empty() {
        return None;
    }
    capture.record(log);
    log.append("source", project.origin_declaration.as_str());
    Some(finalize_simple(
        prompt,
        log,
        "fact_lookup",
        &format!("response:{template}"),
        &body,
        0.88,
    ))
}
