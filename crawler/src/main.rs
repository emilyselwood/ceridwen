use std::fs;
use std::fs::OpenOptions;
use std::io::ErrorKind;

use crate::error::Error;
use ceridwen::config::Config;
use ceridwen::index_sled::Index;
use log::info;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::Logger;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;

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
    let lock_file_result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path);
    let lock_file = match lock_file_result {
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                println!("Already existing crawler lock file at {}. If the previous run crashed remove the file manually", &lock_path.to_string_lossy());
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
        println!("You will probably need to clean this up manually! {lock_path:?}");
    }
}

async fn crawler_main() -> Result<(), Error> {
    println!("Crawler starting. Loading config");
    let config = Config::load()?;

    println!("Config loaded. Setting up logging");
    configure_logging()?;
    info!(
        "Logging setup. Logging level set to {}",
        config.crawler.log_level.clone(),
    );

    let process_start = time::Instant::now();

    let index = Index::load().await?;

    // set up crawler engine...
    // build list of processors to handle.
    info!("Creating {} tasks", config.targets.len());
    let mut tasks = Vec::new();
    for ingester in config.targets.iter() {
        tasks.push(tokio::spawn(ingesters::process_ingester(
            ingester.clone(),
            config.clone(),
            index.clone(),
        )))
    }

    for fut in tasks {
        fut.await?;
    }

    let process_end = process_start.elapsed();
    info!("processing took: {}", process_end);

    Ok(())
}

fn configure_logging() -> Result<Handle, Error> {
    // TODO: config settings for this
    let log_path = ceridwen::system_root().join("logs").join("crawler");

    let window_size = 5;
    let fixed_window_roller = FixedWindowRoller::builder().build(
        log_path.join("crawler_{}.log").to_str().unwrap(),
        window_size,
    )?;

    let size_limit = 50 * 1024 * 1024; // 5MB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);

    let compound_policy =
        CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));

    let file_logger = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .append(true)
        .build(log_path.join("crawler.log"), Box::new(compound_policy))?;

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {h({({l}):5.5})} | {f}:{L} — {m}{n}",
        )))
        .build();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file_logger)))
        // Turn down some dependencies to be less chatty
        .logger(Logger::builder().build("sled", LevelFilter::Info))
        .build(
            Root::builder()
                .appender("file")
                .appender("stdout")
                .build(LevelFilter::Trace),
        )?;

    let handle = log4rs::init_config(config)?;
    Ok(handle)
}
