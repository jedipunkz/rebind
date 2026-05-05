use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    path::PathBuf,
    str::FromStr,
};

use serde::{Deserialize, Deserializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to resolve executable path: {0}")]
    ExePath(std::io::Error),
    #[error("failed to read config {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write default config {path}: {source}")]
    WriteDefault {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_yaml::Error,
    },
    #[error("unsupported config version {0}")]
    UnsupportedVersion(u32),
    #[error("invalid binding `{binding}`: {reason}")]
    InvalidBinding { binding: String, reason: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub enabled: bool,
    ignored_apps: HashSet<String>,
    bindings: BTreeMap<KeyChord, BindingAction>,
}

impl Config {
    pub fn should_ignore_app(&self, exe_name: &str) -> bool {
        self.ignored_apps.contains(&exe_name.to_ascii_lowercase())
    }

    pub fn action_for(&self, chord: &KeyChord) -> Option<&BindingAction> {
        self.bindings.get(chord)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingAction {
    chords: Vec<KeyChord>,
}

impl BindingAction {
    pub fn chords(&self) -> &[KeyChord] {
        &self.chords
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyChord {
    pub modifiers: Modifiers,
    pub key: Key,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Char(char),
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Delete,
    Escape,
    Enter,
    Tab,
    Space,
}

impl FromStr for KeyChord {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err("empty chord".to_string());
        }

        let mut modifiers = Modifiers::default();
        let mut key = None;

        for part in normalized.split('-') {
            match part {
                "ctrl" | "control" | "c" => modifiers.ctrl = true,
                "shift" | "s" => modifiers.shift = true,
                "alt" | "meta" | "m" => modifiers.alt = true,
                "win" | "super" | "cmd" => modifiers.win = true,
                token if key.is_none() => key = Some(parse_key(token)?),
                token => return Err(format!("unexpected token `{token}`")),
            }
        }

        let key = key.ok_or_else(|| "missing key".to_string())?;
        Ok(Self { modifiers, key })
    }
}

fn parse_key(token: &str) -> Result<Key, String> {
    match token {
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "left" => Ok(Key::Left),
        "right" => Ok(Key::Right),
        "up" => Ok(Key::Up),
        "down" => Ok(Key::Down),
        "backspace" | "bs" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "escape" | "esc" => Ok(Key::Escape),
        "enter" | "return" => Ok(Key::Enter),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        single if single.chars().count() == 1 => {
            let ch = single.chars().next().expect("count checked");
            if ch.is_ascii_alphanumeric() {
                Ok(Key::Char(ch))
            } else {
                Err(format!("unsupported character key `{single}`"))
            }
        }
        _ => Err(format!("unknown key `{token}`")),
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    version: u32,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default)]
    ignore_app: Vec<String>,
    #[serde(default)]
    bindings: BTreeMap<String, RawAction>,
}

#[derive(Debug)]
enum RawAction {
    Chord(String),
    Sequence(Vec<String>),
}

impl<'de> Deserialize<'de> for RawAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Chord(String),
            Sequence { sequence: Vec<String> },
        }

        match Repr::deserialize(deserializer)? {
            Repr::Chord(chord) => Ok(Self::Chord(chord)),
            Repr::Sequence { sequence } => Ok(Self::Sequence(sequence)),
        }
    }
}

fn default_enabled() -> bool {
    true
}

pub fn config_path() -> Result<PathBuf, ConfigError> {
    let exe = env::current_exe().map_err(ConfigError::ExePath)?;
    Ok(exe
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rebind.yaml"))
}

pub fn load_or_create_default() -> Result<(PathBuf, Config), ConfigError> {
    let path = config_path()?;
    if !path.exists() {
        fs::write(&path, DEFAULT_CONFIG).map_err(|source| ConfigError::WriteDefault {
            path: path.clone(),
            source,
        })?;
    }
    load_from_path(path)
}

pub fn load_from_path(path: PathBuf) -> Result<(PathBuf, Config), ConfigError> {
    let content = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;
    let raw: RawConfig = serde_yaml::from_str(&content).map_err(|source| ConfigError::Parse {
        path: path.clone(),
        source,
    })?;
    let config = normalize(raw)?;
    Ok((path, config))
}

fn normalize(raw: RawConfig) -> Result<Config, ConfigError> {
    if raw.version != 1 {
        return Err(ConfigError::UnsupportedVersion(raw.version));
    }

    let ignored_apps = raw
        .ignore_app
        .into_iter()
        .map(|name| name.trim().to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .collect();

    let mut bindings = BTreeMap::new();
    for (binding, action) in raw.bindings {
        let source = binding
            .parse::<KeyChord>()
            .map_err(|reason| ConfigError::InvalidBinding {
                binding: binding.clone(),
                reason,
            })?;

        let chords = match action {
            RawAction::Chord(chord) => vec![parse_action_chord(&binding, &chord)?],
            RawAction::Sequence(sequence) => sequence
                .into_iter()
                .map(|chord| parse_action_chord(&binding, &chord))
                .collect::<Result<Vec<_>, _>>()?,
        };
        bindings.insert(source, BindingAction { chords });
    }

    Ok(Config {
        enabled: raw.enabled,
        ignored_apps,
        bindings,
    })
}

fn parse_action_chord(binding: &str, chord: &str) -> Result<KeyChord, ConfigError> {
    chord
        .parse::<KeyChord>()
        .map_err(|reason| ConfigError::InvalidBinding {
            binding: binding.to_string(),
            reason: format!("invalid action `{chord}`: {reason}"),
        })
}

pub const DEFAULT_CONFIG: &str = r#"version: 1
enabled: true

ignore_app:
  - Code.exe
  - WindowsTerminal.exe
  - emacs.exe

bindings:
  ctrl-a: home
  ctrl-e: end
  ctrl-b: left
  ctrl-f: right
  ctrl-p: up
  ctrl-n: down
  ctrl-h: backspace
  ctrl-d: delete
  ctrl-k:
    sequence:
      - shift-end
      - ctrl-x
  ctrl-w: ctrl-x
  ctrl-y: ctrl-v
  ctrl-g: escape
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_config() {
        let raw: RawConfig = serde_yaml::from_str(DEFAULT_CONFIG).unwrap();
        let config = normalize(raw).unwrap();
        assert!(config.enabled);
        assert!(config.should_ignore_app("code.EXE"));
        assert!(config.action_for(&"ctrl-a".parse().unwrap()).is_some());
    }

    #[test]
    fn parses_chord_modifiers() {
        let chord: KeyChord = "ctrl-shift-end".parse().unwrap();
        assert!(chord.modifiers.ctrl);
        assert!(chord.modifiers.shift);
        assert_eq!(chord.key, Key::End);
    }
}
