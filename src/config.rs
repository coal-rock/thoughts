use anyhow::{Result, anyhow};
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct ConfigProto {
    pub vault_path: Option<PathBuf>,
    pub thoughts_path: Option<PathBuf>,
    pub temp_file_path: Option<PathBuf>,
    pub editor_command: Option<String>,
    pub reactive: Option<bool>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
}

impl Default for ConfigProto {
    fn default() -> ConfigProto {
        let temp_file_path = if cfg!(unix) {
            Some(PathBuf::from("/tmp/thought.md"))
        } else if cfg!(windows) {
            Some(PathBuf::from("%temp%\\thought.md"))
        } else {
            None
        };

        ConfigProto {
            vault_path: None,
            thoughts_path: None,
            temp_file_path,
            editor_command: None,
            reactive: Some(true),
            min_width: Some(58),
            min_height: Some(18),
        }
    }
}

struct Config {
    pub vault_path: PathBuf,
    pub thoughts_path: PathBuf,
    pub temp_file_path: PathBuf,
    pub editor_command: String,
    pub reactive: bool,
    pub min_width: u32,
    pub min_height: u32,
}

impl Config {
    // TODO: test this on both Windows and MacOS
    /// Returns the configuration path
    /// - Will default to different values depending on operating system
    /// - Can be overwritten by environmental variables
    pub fn get_path() -> PathBuf {
        let config_path = if cfg!(unix) {
            PathBuf::from("~/.config/thoughts.toml")
        } else if cfg!(windows) {
            PathBuf::from("%USERPROFILE%\\AppData\\Local\\MyApp\\")
        } else {
            panic!("we should not be here.")
        };

        match env::var_os("THOUGHTS_CONFIG_PATH") {
            Some(env_var_path) => return PathBuf::from(env_var_path),
            None => return config_path,
        }
    }

    pub fn read(config_path: PathBuf) -> Result<Config> {
        let config_proto: ConfigProto = Figment::from(Serialized::defaults(ConfigProto::default()))
            .merge(Toml::file(config_path))
            .merge(Env::prefixed("THOUGHTS_"))
            .extract()?;

        Ok(Config {
            vault_path: config_proto.vault_path.ok_or(anyhow!(
                "expected path to Obsidian vault in config file or environmental variable"
            ))?,
            thoughts_path: config_proto.thoughts_path.ok_or(anyhow!(
                "expected path to subfolder within Obsidian vault containing thoughts"
            ))?,
            editor_command: config_proto
                .editor_command
                .ok_or(anyhow!("expected editor command"))?,
            // These should never be anything other than Some(T)
            temp_file_path: config_proto.temp_file_path.unwrap(),
            reactive: config_proto.reactive.unwrap(),
            min_width: config_proto.min_width.unwrap(),
            min_height: config_proto.min_height.unwrap(),
        })
    }
}
