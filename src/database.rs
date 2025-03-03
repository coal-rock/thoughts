// TODO: add a custom tagging system so that the user can create their own tags
// An example of a tag would be "favorite"

use anyhow::Result;
use glob::glob;
use std::fs;
use std::path::PathBuf;

pub struct Entry {
    title: String,
    favorite: bool,
    content: String,
    path: PathBuf,
}

pub struct Database {
    path_str: String,
    entries: Vec<Entry>,
}

impl Database {
    /// Scans the filesystem and stores the state in-memory
    pub fn poll(&mut self) -> Result<()> {
        self.entries.clear();

        for file in glob(&self.path_str)? {
            println!("{:#?}", file);
        }

        Ok(())
    }

    pub fn new(vault_path: PathBuf, thoughts_path: PathBuf) -> Database {
        let mut full_thoughts_path = vault_path;
        full_thoughts_path.push(thoughts_path);

        if !full_thoughts_path.exists() {
            println!("asdajs");
            fs::create_dir_all(&full_thoughts_path).unwrap();
        }

        full_thoughts_path.push("*.md");
        let path_str = full_thoughts_path.to_str().unwrap().to_string();

        Database {
            path_str,
            entries: vec![],
        }
    }
}
