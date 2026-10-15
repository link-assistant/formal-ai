//! Runtime-only cache paths and immutable derivations; explicit legacy APIs stay unchanged.
use crate::derivation::Derivation;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

pub struct RuntimeCachePaths {
    pub runtime_root: PathBuf,
    pub derivation_directory: PathBuf,
    /// Native seeds remain embedded; this resolver never redirects their reads.
    pub read_only_seed_roots: Vec<PathBuf>,
}

fn invalid(reason: &str) -> io::Error {
    io::Error::other(reason)
}

pub fn runtime_cache_paths() -> io::Result<RuntimeCachePaths> {
    let runtime_root = if let Some(explicit) = std::env::var_os("FORMAL_AI_CACHE_DIR") {
        if explicit.to_string_lossy().trim().is_empty() {
            return Err(invalid("runtime_cache_root_invalid"));
        }
        let path = PathBuf::from(explicit);
        if path.is_absolute() { path } else { std::env::current_dir()?.join(path) }
    } else if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME").filter(|value| Path::new(value).is_absolute()) {
        PathBuf::from(xdg).join("formal-ai")
    } else if let Some(local) = std::env::var_os("LOCALAPPDATA").filter(|value| cfg!(windows) && Path::new(value).is_absolute()) {
        PathBuf::from(local).join("formal-ai/Cache")
    } else {
        let home = if cfg!(windows) {
            std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
        } else {
            std::env::var_os("HOME")
        }.filter(|value| Path::new(value).is_absolute()).ok_or_else(|| invalid("runtime_cache_home_unavailable"))?;
        PathBuf::from(home).join(if cfg!(windows) { "AppData/Local/formal-ai/Cache" } else if cfg!(target_os = "macos") { "Library/Caches/formal-ai" } else { ".cache/formal-ai" })
    };
    let runtime_root = normalize_runtime_root(&runtime_root);
    Ok(RuntimeCachePaths {
        derivation_directory: runtime_root.join("derivations"),
        runtime_root,
        read_only_seed_roots: Vec::new(),
    })
}

fn normalize_runtime_root(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => { normalized.pop(); }
            value => normalized.push(value.as_os_str()),
        }
    }
    normalized
}

fn locator_parts(locator: &str) -> io::Result<(&str, Option<&str>)> {
    let mut parts = locator.split('@');
    let answer = parts.next().unwrap_or_default();
    let digest = parts.next();
    if answer.is_empty()
        || !answer.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        || parts.next().is_some()
        || digest.is_some_and(|value| value.len() != 64 || !value.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)))
    {
        return Err(invalid("derivation_answer_id_unusable"));
    }
    Ok((answer, digest))
}

pub struct RuntimeDerivationLocation {
    pub path: PathBuf,
    pub locator: String,
}

static TEMP_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn persist_derivation(record: &Derivation) -> io::Result<RuntimeDerivationLocation> {
    let (answer, selector) = locator_parts(&record.answer_id)?;
    if selector.is_some() {
        return Err(invalid("derivation_answer_id_unusable"));
    }
    let root = runtime_cache_paths()?.derivation_directory;
    let directory = root.join(answer);
    let pending = root.join(".pending");
    for folder in [&directory, &pending] {
        let mut directories = std::fs::DirBuilder::new();
        directories.recursive(true);
        #[cfg(unix)]
        directories.mode(0o700);
        directories.create(folder)?;
        if !std::fs::symlink_metadata(folder)?.file_type().is_dir() {
            return Err(invalid("derivation_directory_unusable"));
        }
    }
    let bytes = record.to_lino().into_bytes();
    let digest = crate::source_fetch::sha256_hex(&bytes);
    let path = directory.join(format!("{digest}.lino"));
    let mut owned_temporary = None;
    for _ in 0..32 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temporary = pending.join(format!("{answer}.{}.{sequence}.tmp", std::process::id()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&temporary) {
            Ok(file) => {
                owned_temporary = Some((temporary, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let (temporary, mut file) = owned_temporary
        .ok_or_else(|| invalid("derivation_temporary_name_unavailable"))?;
    let identity = file.metadata()?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        if std::fs::read(&temporary)? != bytes {
            return Err(invalid("derivation_record_custody_failed"));
        }
        // Both links are on the runtime cache volume; publication cannot replace.
        match std::fs::hard_link(&temporary, &path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        if !std::fs::symlink_metadata(&path)?.file_type().is_file()
            || std::fs::read(&path)? != bytes
        {
            return Err(invalid("derivation_record_custody_failed"));
        }
        Ok(RuntimeDerivationLocation {
            path,
            locator: format!("{answer}@{digest}"),
        })
    })();
    // Remove only the unique name successfully created by this invocation.
    // Unix inode checks additionally refuse namespace replacement before cleanup.
    match std::fs::symlink_metadata(&temporary) {
        Ok(current) => {
            if !current.file_type().is_file() {
                return Err(invalid("derivation_temporary_custody_failed"));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if current.dev() != identity.dev() || current.ino() != identity.ino() {
                    return Err(invalid("derivation_temporary_custody_failed"));
                }
            }
            #[cfg(not(unix))]
            match (current.created(), identity.created()) {
                (Ok(left), Ok(right)) if left == right => {}
                _ => return Err(invalid("derivation_temporary_custody_failed")),
            }
            std::fs::remove_file(&temporary)?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    result
}

pub fn load_derivation(locator: &str) -> io::Result<Option<Derivation>> {
    let (answer, selector) = locator_parts(locator)?;
    let directory = runtime_cache_paths()?.derivation_directory.join(answer);
    let metadata = match std::fs::symlink_metadata(&directory) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_dir() { return Err(invalid("derivation_directory_unusable")); }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&directory)? {
        let name = entry?.file_name().into_string().map_err(|_| invalid("derivation_record_custody_failed"))?;
        let stem = name.strip_suffix(".lino").ok_or_else(|| invalid("derivation_record_custody_failed"))?;
        if stem.len() != 64 || !stem.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
            return Err(invalid("derivation_record_custody_failed"));
        }
        names.push(name);
    }
    if selector.is_none() && names.len() > 1 {
        return Err(invalid(&format!("derivation_record_ambiguous: use {answer}@<record-digest>")));
    }
    let name = match selector {
        Some(value) => format!("{value}.lino"),
        None => match names.first() { Some(value) => value.clone(), None => return Ok(None) },
    };
    if !names.contains(&name) { return Ok(None); }
    let path = directory.join(&name);
    if !std::fs::symlink_metadata(&path)?.file_type().is_file() { return Err(invalid("derivation_record_custody_failed")); }
    let bytes = std::fs::read(path)?;
    if crate::source_fetch::sha256_hex(&bytes) != name.trim_end_matches(".lino") {
        return Err(invalid("derivation_record_custody_failed"));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| invalid("derivation_record_custody_failed"))?;
    let record = Derivation::from_lino(text).ok_or_else(|| invalid("derivation_record_custody_failed"))?;
    if record.answer_id != answer || record.to_lino() != text { return Err(invalid("derivation_record_custody_failed")); }
    Ok(Some(record))
}
