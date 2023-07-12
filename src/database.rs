use crate::search::{tokenize_search, TokenStream};
use crate::tui::Entry;
use std::path::PathBuf;

pub struct EntryQuery {
    pub id: i32,
    pub title: String,
    pub timestamp: String,
    pub content: String,
    pub favorite: bool,
}

pub trait Database {
    fn update_entry(&mut self, entry: EntryQuery);
    fn delete_entry(&mut self, id: i32);
    fn insert_entry(&mut self, entry: EntryQuery);
    fn get_all_entries(&self) -> Vec<Entry>;
    fn backup(&self, backup_dir: &PathBuf);

    fn get_entry_count(&self) -> i32 {
        self.get_all_entries().len().try_into().unwrap()
    }

    fn search(&mut self, query: String) -> Vec<Entry> {
        let mut entries = crate::search::search(self.get_all_entries(), query.clone());

        // this is actually a really pretty hack :3
        // of course, we could tokenize debug seperately,
        // but this is more than fine for a debug feature
        if query.starts_with("debug ") {
            let query = query.replacen("debug ", "", 1);
            entries.push(self.debug_search_tokens(query.clone()));
            entries.push(self.debug_search_parsed(tokenize_search(query)));
        }

        entries
    }

    // Displays the tokenized input of the search query
    fn debug_search_tokens(&self, query: String) -> Entry {
        let tokens = tokenize_search(query);

        let entry = EntryQuery {
            id: -1,
            title: "debug".to_string(),
            timestamp: 0.to_string(),
            content: format!("{:#?}", tokens),
            favorite: true,
        };

        entry.into()
    }

    // Displays the parsed tokens
    fn debug_search_parsed(&self, tokens: TokenStream) -> Entry {
        let entry = EntryQuery {
            id: -2,
            title: "debug".to_string(),
            timestamp: 0.to_string(),
            content: format!("{:#?}", tokens.tokens),
            favorite: true,
        };

        entry.into()
    }
}
