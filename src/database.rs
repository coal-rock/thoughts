// TODO: add a custom tagging system so that the user can create their own tags
// An example of a tag would be "favorite"

use anyhow::{Result, anyhow};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs::{self, read_to_string};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub struct Entry {
    title: String,
    favorite: bool,
    content: String,
    tags: Vec<String>,
    path: PathBuf,
    created_at: u64,
    modified_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Frontmatter {
    favorite: bool,
    thoughts: bool,
    tags: Vec<String>,
}

pub struct Database {
    /// Path to directory containing all Thoughts, followed by trailing `/*.md`
    /// Eg: `/home/coal/Important/Vault/Thoughts/*.md`
    path_str: String,
    entries: Vec<Entry>,
}

impl Database {
    /// Scans the filesystem and stores the state in-memory.
    /// Expects users to call periodically as to minimize FS-related overhead.
    /// TODO: think about implementing checksums/hashes to prevent
    /// excessive re-reads
    ///
    /// TODO: add proper logging for errors with parsing, instead of ignoring them
    pub fn poll(&mut self) -> Result<()> {
        self.entries.clear();

        for file_path in glob(&self.path_str)? {
            let Ok(file_path) = file_path else {
                continue;
            };

            let Ok(parsed_entry) = Database::parse_entry(file_path) else {
                continue;
            };

            println!("{:#?}", parsed_entry);
            self.entries.push(parsed_entry);
        }

        Ok(())
    }

    // The day that I use regex will be cherished by many
    fn parse_entry(file_path: PathBuf) -> Result<Entry> {
        let title = file_path
            .file_stem()
            .ok_or(anyhow!("unable to parse entry name"))?
            .to_str()
            .ok_or(anyhow!("unable to convert path to &str"))?
            .to_string();

        let created_at = fs::metadata(&file_path)?
            .created()?
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let modified_at = fs::metadata(&file_path)?
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let entry_content = read_to_string(&file_path)?;
        let entry_content_lines = entry_content.split('\n').collect::<Vec<&str>>();

        if entry_content_lines.len() < 1 {
            return Err(anyhow!("entry appears to be empty"));
        }

        if entry_content_lines[0] != "---" {
            return Err(anyhow!(
                "entry doesn't appear to contain proper frontmatter"
            ));
        }

        let mut frontmatter_end: Option<usize> = None;

        for (index, value) in entry_content_lines[1..].iter().enumerate() {
            if *value == "---" {
                frontmatter_end = Some(index)
            }
        }

        let Some(frontmatter_end) = frontmatter_end else {
            return Err(anyhow!("frontmatter doesn't appear to properly terminate"));
        };

        let frontmatter_str = entry_content_lines[1..=frontmatter_end]
            .iter()
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n");

        let frontmatter: Frontmatter = serde_yaml::from_str(&frontmatter_str)?;

        let content = entry_content_lines[frontmatter_end + 2..]
            .iter()
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n");

        Ok(Entry {
            title,
            favorite: frontmatter.favorite,
            content,
            tags: frontmatter.tags,
            path: file_path,
            created_at,
            modified_at,
        })
    }

    /// Creates a new `Database`.
    ///
    /// Will scaffold required directories if not already present
    pub fn new(vault_path: PathBuf, thoughts_path: PathBuf) -> Database {
        let mut full_thoughts_path = vault_path;
        full_thoughts_path.push(thoughts_path);

        if !full_thoughts_path.exists() {
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
