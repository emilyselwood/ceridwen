use std::fs;

use ceridwen::config;
use ceridwen::index;
use ceridwen::index::Index;

fn main() {
    // TODO: Refuse to over write stuff that already exists.
    // TODO: Flag parameter to allow overwriting stuff and resetting everything to default.
    println!("Welcome! Setting up Ceridwen.");

    let root_dir = ceridwen::system_root();

    println!("Creating root dir: {}", root_dir.to_str().unwrap());

    let result = fs::create_dir_all(root_dir.to_path_buf());
    if result.is_err() {
        println!(
            "Could not create ceridwen directory at {:?}: {}",
            root_dir,
            result.unwrap_err(),
        );
        return;
    }

    let config_path = config::Config::config_path();
    println!("Creating base config at {config_path}");

    let new_config = config::Config::new();

    // TODO: ask about any user configurable settings

    let result = new_config.save();
    if result.is_err() {
        println!(
            "Could not create config file at {}: {}",
            config_path,
            result.unwrap_err(),
        );
        return;
    }

    // Create the empty index.
    let index = Index::new(&index::index_dir());
    if index.is_err() {
        println!("Could not create index! {:?}", index.unwrap_err());
        return;
    }
}
