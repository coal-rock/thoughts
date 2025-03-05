use anyhow::{Result, anyhow};
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
struct ConfigProto {
    pub thoughts_path: Option<PathBuf>,
    pub temp_file_path: Option<PathBuf>,
    pub editor_command: Option<String>,
    pub reactive: Option<bool>,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub react_width: Option<u16>,
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
            thoughts_path: None,
            temp_file_path,
            editor_command: None,
            reactive: Some(true),
            min_width: Some(58),
            min_height: Some(18),
            react_width: Some(80),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub thoughts_path: PathBuf,
    pub temp_file_path: PathBuf,
    pub editor_command: String,
    pub reactive: bool,
    pub min_width: u16,
    pub min_height: u16,
    pub react_width: u16,
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
            PathBuf::from("%USERPROFILE%\\AppData\\Local\\Thoughts\\thoughts.toml")
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
            .merge(Toml::file("thoughts.toml"))
            .merge(Env::prefixed("THOUGHTS_"))
            .extract()?;

        Ok(Config {
            thoughts_path: Config::expand_path(
                config_proto.thoughts_path,
                "expected path to thoughts directory",
            )?,
            editor_command: config_proto
                .editor_command
                .ok_or(anyhow!("expected editor command"))?,
            // These should never be anything other than Some(T)
            temp_file_path: Config::expand_path(
                config_proto.temp_file_path,
                "expected path to temp file",
            )?,
            reactive: config_proto.reactive.unwrap(),
            min_width: config_proto.min_width.unwrap(),
            min_height: config_proto.min_height.unwrap(),
            react_width: config_proto.react_width.unwrap(),
        })
    }

    /// Takes in an Option<PathBuf> and returns either an error,
    /// or an appropriately expanded, OS-specific PathBuf
    /// eg:
    ///     On UNIX, `~/.config/` becomes `/home/user/.config/`
    fn expand_path(path: Option<PathBuf>, if_none: &str) -> Result<PathBuf> {
        let if_none = if_none.to_string();
        let path = path.ok_or(anyhow!(if_none))?;

        let path_str = path
            .to_str()
            .ok_or(anyhow!("unable to cast path to string"))?;

        let expanded_path = shellexpand::full(path_str)?.to_string();

        Ok(PathBuf::from(expanded_path))
    }
}
