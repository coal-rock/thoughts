// TODO: add a custom tagging system so that the user can create their own tags
// An example of a tag would be "favorite"

use anyhow::{Result, anyhow};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, read_to_string};
use std::io::Write;
use std::ops::Not;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Debug, Default, Clone)]
pub struct Entry {
    pub title: String,
    pub favorite: bool,
    pub content: String,
    pub tags: Vec<String>,
    pub path: PathBuf,
    pub created_at: u64,
    pub modified_at: u64,
}

impl Into<Frontmatter> for &Entry {
    fn into(self) -> Frontmatter {
        Frontmatter {
            favorite: self.favorite,
            tags: self.tags.clone(),
        }
    }
}

// This whole struct is just Serde wizardry
#[derive(Serialize, Deserialize, Debug)]
struct Frontmatter {
    #[serde(skip_serializing_if = "<&bool>::not")]
    #[serde(default)]
    favorite: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Database {
    /// Path to directory containing all Thoughts, followed by trailing `/*.md`
    /// Eg: `/home/coal/Important/Vault/Thoughts/*.md`
    path_str: String,
    pub entries: Vec<Entry>,
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

        eprintln!("{}", &self.path_str);
        for file_path in glob(&self.path_str)? {
            let Ok(file_path) = file_path else {
                continue;
            };

            let parsed_entry = Database::parse_entry(file_path.clone());

            let parsed_entry = match parsed_entry {
                Ok(parsed_entry) => parsed_entry,
                Err(err) => {
                    println!("err parsing: {}", err);
                    continue;
                }
            };

            //
            // let Ok(parsed_entry) = Database::parse_entry(file_path.clone()) else {
            //     continue;
            // };

            // self.write_entry(&parsed_entry);
            self.entries.push(parsed_entry);
        }

        Ok(())
    }

    // The day that I use regex will be cherished by many
    fn parse_entry(file_path: PathBuf) -> Result<Entry> {
        if !file_path.is_file() {
            return Err(anyhow!("tried to parse a non-file"));
        }

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

        eprintln!("reading file: {}", file_path.to_str().unwrap());
        let entry_content = read_to_string(&file_path)?;
        let entry_content_lines = entry_content.split('\n').collect::<Vec<&str>>();

        if entry_content_lines.len() == 0 {
            return Err(anyhow!("entry appears to be empty"));
        }

        let (frontmatter, content_start) = match Database::parse_frontmatter(&entry_content_lines)?
        {
            Some((frontmatter, content_start)) => (frontmatter, content_start),
            None => (
                Frontmatter {
                    favorite: false,
                    tags: vec![],
                },
                0,
            ),
        };

        let content = entry_content_lines[content_start..]
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

    /// Writes entry to file. Will overwrite if file already exists
    pub fn write_entry(&self, entry: &Entry) -> Result<()> {
        let mut output = String::new();

        output.push_str("---\n");

        let frontmatter: Frontmatter = entry.into();
        let frontmatter_str = serde_yaml::to_string(&frontmatter)?;

        output.push_str(frontmatter_str.as_str());
        output.push_str("---\n");

        output.push_str(&entry.content);

        let mut file = File::create(&entry.path)?;
        file.write_all(output.as_bytes())?;

        Ok(())
    }

    /// Consumes a file split at newlines characters, and returns a `Result<Option<T>>`
    ///
    /// Result will be Err if frontmatter was detected, but was unable to be parsed,
    /// if no frontmatter was found, then this function will return `Ok(None)`,
    /// if frontmatter was found, and was parsed properly, then the function will return:
    /// `Ok(Some((Frontmatter, ContentStartIndex)))`
    fn parse_frontmatter(entry_content_lines: &Vec<&str>) -> Result<Option<(Frontmatter, usize)>> {
        if entry_content_lines[0] != "---" {
            return Ok(None);
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

        eprintln!("{}", frontmatter_str);
        let frontmatter: Frontmatter = serde_yaml::from_str(&frontmatter_str)?;

        Ok(Some((frontmatter, frontmatter_end + 2)))
    }

    /// Creates a new `Database`.
    ///
    /// Will scaffold required directories if not already present
    pub fn new(mut thoughts_path: PathBuf) -> Database {
        if !thoughts_path.exists() {
            fs::create_dir_all(&thoughts_path).unwrap();
        }

        thoughts_path.push("**/*.md");
        let path_str = thoughts_path.to_str().unwrap().to_string();

        let mut database = Database {
            path_str,
            entries: vec![],
        };

        // TODO: a failed poll on new is an irrecoverable state.
        // Pretty print errors, please
        database.poll().unwrap();

        database
    }
}
