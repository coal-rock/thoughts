use figment::providers::{Format, Serialized, Toml};
use figment::Figment;

use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use toml;

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

// I would like to deprecate Sql DB

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DbType {
    Sql,
    Fs,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub db_path: PathBuf,
    pub db_type: DbType,
    pub backup_path: PathBuf,
    pub temp_file_path: PathBuf,
    pub editor_command: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            db_type: DbType::Fs,
            db_path: String::from(tilde("~/.config/thoughts/entries/")).into(),
            backup_path: String::from(tilde("~/.config/thoughts/backups/")).into(),
            temp_file_path: String::from(tilde("~/.config/thoughts/.temp/")).into(),
            editor_command: String::from("nvim"),
        }
    }
}

impl Config {
    fn serialize(&self) -> String {
        toml::to_string(self).unwrap()
    }
}

// TODO: add proper error handling for the love of GOD
// TODO: check to ensure config is valid (paths are real, etc)
pub fn load_config(config_path: String) -> Option<Config> {
    let config_path = PathBuf::from(tilde(&config_path).to_string());

    let config: Option<Config> = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(config_path))
        .extract()
        .unwrap();

    match config {
        // Hack?
        // FIXME:
        Some(mut config) => {
            config.db_path = tilde::<str>(&config.db_path.to_str().unwrap())
                .to_string()
                .into();
            config.backup_path = tilde(&config.backup_path.to_str().unwrap())
                .to_string()
                .into();

            Some(config)
        }
        None => None,
    }
}

pub fn scaffold_config_dir(config: &Config) {
    let config_dir = config.backup_path.parent().unwrap();
    _ = fs::create_dir(config_dir);
    _ = fs::create_dir(config.backup_path.clone());
    _ = fs::create_dir(config.temp_file_path.clone());

    match config.db_type {
        DbType::Sql => {}
        DbType::Fs => _ = fs::create_dir(&config.db_path),
    }

    let config_path = config_dir.join("config.toml");
    let mut file = File::create(config_path).unwrap();
    write!(file, "{}", config.serialize()).unwrap();
}
