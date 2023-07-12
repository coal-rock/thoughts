use std::path::PathBuf;

use glob::glob;

use crate::database::Database;
use crate::database::EntryQuery;
use crate::tui::Entry;

pub struct Filesystem {
    path: String,
}

impl Filesystem {
    fn new(path: String) -> Filesystem {
        Filesystem { path }
    }
}

impl Database for Filesystem {
    fn update_entry(&mut self, entry: EntryQuery) {
        todo!()
    }

    fn delete_entry(&mut self, id: i32) {
        todo!()
    }

    fn insert_entry(&mut self, entry: EntryQuery) {
        todo!()
    }

    fn get_all_entries(&self) -> Vec<Entry> {
        let mut entries = Vec::new();

        for file in glob(&format!("{}/*.md", self.path)).unwrap() {
            let file = file.unwrap();
            let file = file.to_str().unwrap();

            let parts: Vec<&str> = file.splitn(1, '_').collect();

            entries.push(
                EntryQuery {
                    title: parts.get(1).unwrap().to_string(),
                    content: todo!(),
                    id: parts.get(0).unwrap().parse().unwrap(),
                    favorite: todo!(),
                    timestamp: todo!(),
                }
                .into(),
            );
        }

        entries
    }

    fn backup(&self, backup_dir: &PathBuf) {
        todo!()
    }
}
