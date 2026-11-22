//! Release-note selection shared by the publishing CLI and native contracts.
//! Fence scanning follows the Unlicense Rust pipeline template,
//! 5f1e7a96acbf7a93b567195443cf6e5158fbbf1f, scripts/release-content.rs.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[allow(dead_code)]
fn main() {}

fn markdown_line(line: &str) -> Option<&str> {
    let trimmed = line.trim_start_matches(' ');
    (line.len() - trimmed.len() <= 3).then_some(trimmed)
}

fn release_heading(line: &str) -> Option<&str> {
    let (version, suffix) = line.strip_prefix("## [")?.split_once(']')?;
    if version.is_empty()
        || !suffix
            .as_bytes()
            .first()
            .is_none_or(|b| matches!(b, b' ' | b'\t'))
    {
        return None;
    }
    Some(version)
}

/// Byte offsets of actual release headings, ignoring fenced examples.
fn headings(content: &str) -> impl Iterator<Item = (usize, usize, &str)> {
    let mut fence: Option<(u8, usize)> = None;
    let mut offset = 0;
    content.split_inclusive('\n').filter_map(move |full_line| {
        let start = offset;
        offset += full_line.len();
        let line = markdown_line(full_line.trim_end_matches(['\r', '\n']))?;
        let marker = line.as_bytes().first().copied();
        let width = line
            .as_bytes()
            .iter()
            .take_while(|&&b| Some(b) == marker)
            .count();
        if let Some((character, minimum)) = fence {
            if marker == Some(character)
                && width >= minimum
                && line[width..].bytes().all(|b| matches!(b, b' ' | b'\t'))
            {
                fence = None;
            }
            return None;
        }
        if matches!(marker, Some(b'`' | b'~'))
            && width >= 3
            && (marker != Some(b'`') || !line[width..].contains('`'))
        {
            fence = Some((marker.unwrap(), width));
            return None;
        }
        release_heading(line).map(|version| (start, offset, version))
    })
}

/// The collector inserts before the first actual release without rewriting old bytes.
#[allow(dead_code)] // Used by the collectors; the publishing CLI selects sections only.
pub fn first_heading_offset(content: &str) -> Option<usize> {
    headings(content).next().map(|(start, _, _)| start)
}

/// Ignore release-looking headings inside backtick or tilde fenced examples.
pub fn section(content: &str, version: &str) -> Option<String> {
    let mut body_start = None;
    for (start, body, found) in headings(content) {
        if let Some(body) = body_start {
            return Some(notes_or_fallback(&content[body..start], version));
        }
        if found == version {
            body_start = Some(body);
        }
    }
    body_start.map(|body| notes_or_fallback(&content[body..], version))
}

fn notes_or_fallback(body: &str, version: &str) -> String {
    if body.trim().is_empty() {
        format!("Release v{version}")
    } else {
        body.trim().to_owned()
    }
}

fn context(operation: &str, path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!(
            "{operation} release changelog '{}': {error}",
            path.display()
        ),
    )
}

fn read_optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(context("read", path, error)),
    }
}

fn archives(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(context("list", directory, error)),
    };
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| context("list entry in", directory, error))?;
        let name = entry.file_name().into_string().map_err(|name| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid UTF-8 changelog archive name: {name:?}"),
            )
        })?;
        if let Some(version) = name
            .strip_prefix("releases-from-")
            .and_then(|v| v.strip_suffix(".md"))
        {
            let order = version
                .split(['.', '-'])
                .map(|part| part.parse::<u64>().unwrap_or(0))
                .collect::<Vec<_>>();
            files.push((order, entry.path()));
        }
    }
    files.sort();
    files.reverse();
    Ok(files.into_iter().map(|(_, path)| path).collect())
}

/// Absent optional sources retain the generic note; unreadable observations fail.
pub fn for_version(root: &Path, version: &str) -> io::Result<String> {
    if let Some(content) = read_optional(&root.join("CHANGELOG.md"))?
        && let Some(notes) = section(&content, version)
    {
        return Ok(notes);
    }
    for path in archives(&root.join("docs/changelog"))? {
        // An enumerated source is required even if it disappears before read.
        let content = fs::read_to_string(&path).map_err(|error| context("read", &path, error))?;
        if let Some(notes) = section(&content, version) {
            return Ok(notes);
        }
    }
    Ok(format!("Release v{version}"))
}

#[cfg(test)]
#[path = "release-changelog/tests.rs"]
mod tests;
