use super::*;

/// The verdict lines a finished recipe's answer carries, or an empty string
/// when every clause of `objective` is discharged or inapplicable. The
/// candidate is the recipe's files and the last non-empty verification
/// output, compared at its tail.
#[must_use]
#[allow(clippy::literal_string_with_formatting_args)]
pub fn acceptance_report(objective: &str, recipe: &ExecutionRecipe, outputs: &[String]) -> String {
    let specification = specify_task(objective);
    let stdout = outputs
        .iter()
        .rev()
        .find(|output| !output.trim().is_empty())
        .map_or("", String::as_str);
    let mut files = vec![json!({ "path": recipe.path, "source": recipe.source })];
    files.extend(
        recipe
            .supporting_files
            .iter()
            .map(|file| json!({ "path": file.path, "source": file.source })),
    );
    let report = check_task(
        &specification,
        &json!({
            "task_identity": specification["task_identity"],
            "program_path": recipe.path,
            "files": files,
            "observations": [{ "args": [], "stdout": stdout, "tail": true }],
        }),
    );
    let open: Vec<&Value> = report["verdicts"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|entry| entry["verdict"] != "discharged" && entry["verdict"] != "inapplicable")
        .collect();
    // A clause outside the supported subset is a reader gap, not evidence against
    // the candidate: the report is appended only when some verdict is actionable
    // (contradicted, or unresolved for want of an observation).
    let actionable = open
        .iter()
        .any(|entry| !UNREAD.contains(&entry["justification"].as_str().unwrap_or("")));
    if !actionable {
        return String::new();
    }
    let counts: Vec<(String, String)> =
        ["discharged", "contradicted", "unresolved", "inapplicable"]
            .iter()
            .map(|name| (format!("{{{name}}}"), report[*name].to_string()))
            .collect();
    let count_pairs: Vec<(&str, &str)> = counts
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let mut lines = vec![fill("task-acceptance-heading", &count_pairs)];
    for entry in open {
        let span = &entry["span"];
        let (start, end) = (span[0].to_string(), span[1].to_string());
        lines.push(fill(
            "task-acceptance-line",
            &[
                ("{verdict}", entry["verdict"].as_str().unwrap_or("")),
                (
                    "{justification}",
                    entry["justification"].as_str().unwrap_or(""),
                ),
                ("{start}", &start),
                ("{end}", &end),
                (
                    "{clause}",
                    &entry["clause"].as_str().unwrap_or("").replace('\n', " "),
                ),
            ],
        ));
    }
    let closing = if report["complete"] == true {
        "task-acceptance-review"
    } else {
        "task-acceptance-incomplete"
    };
    lines.push(fill(closing, &[]));
    lines.join("\n")
}
