use std::path::{Path, PathBuf};

use crate::seed::ClientIntegration;

use super::user_home_dir;

pub(super) fn resolve_integration_command(integration: &ClientIntegration) -> PathBuf {
    if !integration.command_env.is_empty() {
        if let Some(command) =
            std::env::var_os(&integration.command_env).filter(|value| !value.is_empty())
        {
            return PathBuf::from(command);
        }
    }

    let primary = PathBuf::from(&integration.command);
    if command_available(&primary) {
        return primary;
    }

    integration
        .platform_commands
        .iter()
        .filter(|candidate| candidate.key == std::env::consts::OS)
        .map(|candidate| PathBuf::from(expand_command_path(&candidate.value)))
        .find(|candidate| command_available(candidate))
        .unwrap_or(primary)
}

fn expand_command_path(value: &str) -> String {
    const HOME_PLACEHOLDER: &str = "{home}";
    const LOCAL_APP_DATA_PLACEHOLDER: &str = "{local_app_data}";
    let home = user_home_dir().map_or_else(|_| String::new(), |path| path.display().to_string());
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| (!home.is_empty()).then(|| PathBuf::from(&home).join("AppData/Local")))
        .map_or_else(String::new, |path| path.display().to_string());
    value
        .replace(HOME_PLACEHOLDER, &home)
        .replace(LOCAL_APP_DATA_PLACEHOLDER, &local_app_data)
}

fn command_available(command: &Path) -> bool {
    if command.is_absolute() || command.components().count() > 1 {
        return command.is_file();
    }
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|directory| {
            let candidate = directory.join(command);
            candidate.is_file() || (cfg!(windows) && candidate.with_extension("exe").is_file())
        })
    })
}
