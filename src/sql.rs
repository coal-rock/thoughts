use std::{fs, path::PathBuf};

use crate::{
    config::Config,
    database::{Database, EntryQuery},
    tui::Entry,
};
use rusqlite::Connection;

pub struct SqlDatabase {
    connection: Connection,
}

impl SqlDatabase {
    pub fn new(config: &Config) -> SqlDatabase {
        let connection: Connection = Connection::open(&config.db_path).unwrap();

        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS entries(
                    id integer primary key autoincrement,
                    title text not null,
                    timestamp text not null,
                    content text not null,
                    favorite boolean not null
                )",
                (),
            )
            .unwrap();

        SqlDatabase { connection }
    }
}

// TODO: don't hardcode this path
// TODO: maybe offload searching to sqlite
impl Database for SqlDatabase {
    fn update_entry(&mut self, entry: EntryQuery) {
        self.connection.execute("UPDATE entries SET title = ?1, timestamp = ?2, content = ?3, favorite = ?4 where id = ?5", (
            &entry.title,
            &entry.timestamp,
            &entry.content,
            &entry.favorite,
            &entry.id,
        )).unwrap();
    }

    fn delete_entry(&mut self, id: i32) {
        self.connection
            .execute("DELETE FROM entries WHERE id = (?1)", &[&id.to_string()])
            .unwrap();
    }

    fn insert_entry(&mut self, entry: EntryQuery) {
        self.connection
            .execute(
                "INSERT INTO entries (title, timestamp, content, favorite) values (?1, ?2, ?3, ?4)",
                (
                    &entry.title,
                    &entry.timestamp,
                    &entry.content,
                    entry.favorite,
                ),
            )
            .unwrap();
    }

    fn get_all_entries(&self) -> Vec<Entry> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, title, timestamp, content, favorite FROM entries")
            .unwrap();

        let entry_iter = stmt
            .query_map([], |row| {
                Ok(EntryQuery {
                    id: row.get(0).unwrap(),
                    title: row.get(1).unwrap(),
                    timestamp: row.get(2).unwrap(),
                    content: row.get(3).unwrap(),
                    favorite: row.get(4).unwrap(),
                })
            })
            .unwrap();

        entry_iter.map(|e| Entry::from(e.unwrap())).collect()
    }

    fn backup(&self, backup_dir: &PathBuf) {
        let backup_num = fs::read_dir(backup_dir).unwrap().count();

        _ = fs::copy(
            self.connection.path().unwrap(),
            backup_dir.join(format!("backup{}.db", backup_num)),
        )
        .unwrap();
    }
}
