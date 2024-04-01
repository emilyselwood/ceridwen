use std::fs;
use std::fs::File;
use std::io::ErrorKind;

use crate::error::Error;
use ceridwen::config::Config;
use env_logger::Env;
use log::info;

mod error;
mod ingesters;
mod robots_text;
mod web_client;

#[tokio::main]
async fn main() {
    // create a lock file, if it already exists then another copy of the crawler is probably running.
    // There is no communication between crawlers, so if two run they do the same thing repeatedly which is a waste.
    // Also there is a good chance of two crawlers trying to write to the same index file.
    let lock_path = ceridwen::system_root().join("crawler.lock");
    let lock_file_result = File::create(&lock_path);
    let lock_file = match lock_file_result {
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                println!("Already existing crawler lock file at {:?}. If the previous run crashed remove the file manually", &lock_path);
                return;
            } else {
                println!("Error obtaining lock file: {:?} {}", &lock_path, e);
                return;
            }
        }
        Ok(f) => f,
    };

    // Run the application and handle error responses
    let result = crawler_main().await;
    if result.is_err() {
        let err = result.unwrap_err();
        println!("Error running crawler: {}", err);
    }

    // Clean up the lock file
    drop(lock_file);
    // If this doesn't work we cant do much about it, but tell the user
    if let Err(e) = fs::remove_file(&lock_path) {
        println!("Could not delete lock file: {e}");
        println!("You will probably need to clean this up manually!");
    }
}

async fn crawler_main() -> Result<(), Error> {
    println!("Crawler starting. Loading config");
    let config = Config::load()?;

    println!("Config loaded. Setting up logging");
    env_logger::init_from_env(Env::default().default_filter_or(config.crawler.log_level.clone()));
    info!(
        "Logging setup. Logging level set to {}",
        config.crawler.log_level.clone(),
    );

    let process_start = time::Instant::now();

    // set up crawler engine...
    // build list of processors to handle.
    info!("Creating {} tasks", config.targets.len());
    let mut tasks = Vec::new();
    for ingester in config.targets.iter() {
        tasks.push(tokio::spawn(ingesters::process_ingester(
            ingester.clone(),
            config.clone(),
        )))
    }

    for fut in tasks {
        fut.await?;
    }

    let process_end = process_start.elapsed();
    info!("processing took: {}", process_end);

    Ok(())
}
