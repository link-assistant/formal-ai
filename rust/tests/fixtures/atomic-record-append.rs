//! Physical fixture provider for the explicit append contract, not generic Write replacement.
use serde_json::{Value, json};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);
const MAXIMUM_BYTES: usize = 65_536;

pub struct AppendWorkspace {
    root: PathBuf,
}
impl AppendWorkspace {
    #[must_use]
    pub fn new() -> Self {
        let number = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("formal-ai-append-{}-{number}", std::process::id()));
        fs::create_dir(&root).expect("create exclusive append fixture workspace");
        Self { root }
    }
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        append(&self.root, args).map_err(|error| error.to_string())
    }
}
impl Default for AppendWorkspace {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for AppendWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
struct AppendLock(PathBuf);
impl Drop for AppendLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

fn append(root: &Path, args: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let field = |name: &str| {
        args.get(name)
            .and_then(Value::as_str)
            .ok_or("missing append field")
    };
    let relative = field("path")?;
    let content = field("content")?;
    let identity = field("record_id")?;
    let request_identity = field("append_request_id")?;
    if field("append_mode")? != "atomic_record_append"
        || content.split('\n').next() != Some("general_change_plan")
        || content.split('\n').nth(1) != Some(identity)
        || content.split('\n').filter(|line| *line == identity).count() != 1
        || content
            .split('\n')
            .filter(|line| *line == "general_change_plan")
            .count()
            != 1
        || !content.ends_with('\n')
        || Path::new(relative)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("invalid append contract or path".into());
    }
    if relative.contains(['\\', ':'])
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err("append path escape or nonportable component".into());
    }
    let physical_root = root.canonicalize()?;
    let target = physical_root.join(relative);
    let parent = target.parent().ok_or("missing append parent")?;
    let mut existing = parent;
    while !existing.exists() {
        existing = existing.parent().ok_or("missing physical root")?;
    }
    if !existing.canonicalize()?.starts_with(&physical_root) {
        return Err("append symlink escape".into());
    }
    fs::create_dir_all(parent)?;
    if parent.canonicalize()? != parent {
        return Err("nonphysical append parent".into());
    }
    let mut lock = target.clone().into_os_string();
    lock.push(".append-lock");
    let lock = PathBuf::from(lock);
    fs::create_dir(&lock)?;
    let _lock = AppendLock(lock);
    let before = match fs::symlink_metadata(&target) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err("append nonregular target".into());
            }
            String::from_utf8(fs::read(&target)?)?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    if before.len() > MAXIMUM_BYTES {
        return Err("append prior bytes exceed contract".into());
    }
    let separator = if !before.is_empty() && !before.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    let mut after = format!("{before}{separator}{content}");
    let operation = if before.split('\n').any(|line| line == identity) {
        if before.split('\n').filter(|line| *line == identity).count() != 1
            || !before.match_indices(content).any(|(start, _)| {
                let following = &before[start + content.len()..];
                (start == 0 || before.as_bytes().get(start - 1) == Some(&b'\n'))
                    && (following.is_empty() || following.starts_with("general_change_plan\n"))
            })
        {
            return Err("append identity collision".into());
        }
        after.clone_from(&before);
        "already_present"
    } else {
        "appended"
    };
    if after.len() > MAXIMUM_BYTES {
        return Err("append result exceeds contract".into());
    }
    if operation == "appended" {
        let mut file = OpenOptions::new().create(true).append(true).open(&target)?;
        let addition = format!("{separator}{content}");
        let written = file.write(addition.as_bytes())?;
        if written != addition.len() {
            return Err("incomplete physical append".into());
        }
        file.sync_all()?;
    }
    let actual = fs::read(&target)?;
    if actual != after.as_bytes() {
        return Err("append readback mismatch".into());
    }
    Ok(json!({"schema":"atomic-record-append/v1","path":relative,
        "record_id":identity,"append_request_id":request_identity,"content":content,
        "before":before,"after":after,"before_bytes":before.len(),"after_bytes":actual.len(),
        "operation":operation,"complete":true,"success":true}))
}
