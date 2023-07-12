use crate::{config::Config, database::EntryQuery, tui::Entry};

use std::{
    fs::{self, remove_file, File},
    io::{Read, Write},
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::NaiveDateTime;
use glob::glob;
use shellexpand::tilde;

fn get_feeling_path() -> String {
    let tilde = tilde("~/.config/feelings/").to_string();
    let path = Path::new(&tilde);

    let mut entry_number = 0;

    for file in path.read_dir().unwrap() {
        if file
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .starts_with("FEELING")
        {
            entry_number += 1;
        }
    }

    path.join(if entry_number == 0 {
        String::from("FEELING.md")
    } else {
        format!("FEELING{:?}.md", entry_number)
    })
    .to_str()
    .unwrap()
    .to_string()
}

// kinda hacky, works for now tho
pub fn post_process(content: String, timestamp: String) -> String {
    let dt = NaiveDateTime::from_timestamp_opt(timestamp.parse().unwrap(), 0).unwrap();
    let content = content.replace("<DATE>", &dt.format("%m-%d-%Y").to_string());
    let content = content.replace("<TIME>", &dt.format("%I:%M%P").to_string());
    content
}

fn get_timestamp() -> String {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        // FIXME: this is a collosal hack.
        - (4 * 60 * 60))
        .to_string()
}

pub fn purge_entry_files(config: &Config) {
    for file in glob(&format!(
        "{}/FEELING*.md",
        config.temp_file_path.to_str().unwrap()
    ))
    .unwrap()
    {
        remove_file(file.unwrap()).unwrap();
    }
}

pub fn new(name: String, editor_cmd: &String) -> Entry {
    let file_path = get_feeling_path();

    {
        let mut file = File::create(&file_path).expect("FS error");
        file.write(&vec![]).unwrap();

        Command::new(editor_cmd)
            .arg(&file_path)
            .spawn()
            .expect("Editor not found.")
            .wait()
            .unwrap();
    }

    let mut file = File::open(&file_path).expect("FS error");

    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();
    fs::remove_file(file_path).unwrap();

    let timestamp = get_timestamp();

    let entry = EntryQuery {
        id: 0,
        title: name,
        timestamp: timestamp.clone(),
        content: post_process(content.clone(), timestamp),
        favorite: false,
    };

    entry.into()
}

// TODO: make this non-hardcoded, fix the ugly unwraps
// TODO: fix weird use of id globally pls
// TODO: database.rs should be the only thing touching id
pub fn edit(entry: Entry, editor_cmd: &String) -> Entry {
    let file_path = get_feeling_path();

    let mut file = File::create(&file_path).expect("FS error");

    file.write_all(&entry.content.clone().into_bytes()).unwrap();

    Command::new(editor_cmd)
        .arg(&file_path)
        .spawn()
        .expect("Editor not found.")
        .wait()
        .unwrap();

    let mut file = File::open(&file_path).expect("FS error");

    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();
    fs::remove_file(file_path).unwrap();

    let entry = EntryQuery {
        id: entry.id,
        title: entry.title,
        timestamp: entry.dt.timestamp().to_string(),
        content: post_process(content.clone(), get_timestamp()),
        favorite: entry.favorite,
    };

    entry.into()
}
