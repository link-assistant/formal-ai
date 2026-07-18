use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::PathBuf;

pub const MEMORY_PATH_ENV: &str = "FORMAL_AI_MEMORY_PATH";
pub const MEMORY_DIRECTORY_NAME: &str = ".formal-ai";
pub const MEMORY_FILE_NAME: &str = "memory.lino";

#[must_use]
pub fn resolve_memory_path_from(
    memory_path: Option<&OsStr>,
    home: Option<&OsStr>,
    app_data: Option<&OsStr>,
    windows: bool,
) -> PathBuf {
    if let Some(path) = memory_path
        .filter(|value| !value.to_string_lossy().trim().is_empty())
        .map(PathBuf::from)
    {
        return path;
    }

    if windows {
        return app_data
            .or(home)
            .map_or_else(|| PathBuf::from("."), PathBuf::from)
            .join("formal-ai")
            .join(MEMORY_FILE_NAME);
    }

    home.map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join(MEMORY_DIRECTORY_NAME)
        .join(MEMORY_FILE_NAME)
}

#[must_use]
pub fn shared_memory_path() -> PathBuf {
    resolve_memory_path_from(
        std::env::var_os(MEMORY_PATH_ENV).as_deref(),
        std::env::var_os("HOME").as_deref(),
        std::env::var_os("APPDATA").as_deref(),
        cfg!(windows),
    )
}

pub fn ensure_shared_memory_file(path: &std::path::Path) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        let existed = parent.exists();
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        if !existed {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }
    OpenOptions::new().create(true).append(true).open(path)?;
    Ok(())
}
