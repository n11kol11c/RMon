use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ConfigFile {
    pub theme: Option<ThemeSection>,
    pub monitor: Option<MonitorSection>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ThemeSection {
    pub name: Option<String>,
    pub colors: Option<ThemeColors>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct ThemeColors {
    pub bg: Option<String>,
    pub panel: Option<String>,
    pub accent: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub dim: Option<String>,
    pub ok: Option<String>,
    pub warn: Option<String>,
    pub danger: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct MonitorSection {
    pub refresh_ms: Option<u64>,
    pub sort: Option<String>,
    pub max_processes: Option<usize>,
}

#[derive(Default)]
pub struct Config {
    pub file: Option<ConfigFile>,
    pub path: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match toml::from_str::<ConfigFile>(&contents) {
            Ok(file) => Self {
                file: Some(file),
                path: Some(path),
            },
            Err(err) => {
                eprintln!("rmon: failed to parse config {}: {err}", path.display());
                Self::default()
            }
        }
    }
}

fn config_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("RMON_CONFIG") {
        return Some(PathBuf::from(p));
    }
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("rmon").join("config.toml"))
    }
    #[cfg(not(windows))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("rmon").join("config.toml"));
        }
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join(".config").join("rmon").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let toml = r##"
[theme]
name = "dracula"

[theme.colors]
accent = "#00ff00"

[monitor]
refresh_ms = 500
sort = "name"
max_processes = 100
"##;
        let cfg: ConfigFile = toml::from_str(toml).unwrap();
        let theme = cfg.theme.unwrap();
        assert_eq!(theme.name.as_deref(), Some("dracula"));
        assert_eq!(theme.colors.unwrap().accent.as_deref(), Some("#00ff00"));
        let mon = cfg.monitor.unwrap();
        assert_eq!(mon.refresh_ms, Some(500));
        assert_eq!(mon.sort.as_deref(), Some("name"));
        assert_eq!(mon.max_processes, Some(100));
    }

    #[test]
    fn empty_config_parses() {
        let cfg: ConfigFile = toml::from_str("").unwrap();
        assert!(cfg.theme.is_none());
        assert!(cfg.monitor.is_none());
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let cfg: ConfigFile = toml::from_str("future = true\n[theme]\nunknown = 1").unwrap();
        assert!(cfg.theme.is_some());
    }
}
