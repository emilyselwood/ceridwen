pub mod config;
pub mod crawler;
pub mod data;
pub mod error;
pub mod index_sled;
pub mod server;
pub mod utils;

use std::fs;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::str::FromStr;

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

use crate::config::Config;
use crate::error::Error;

#[tokio::main]
async fn main() {
    // create a lock file, if it already exists then another copy of ceridwen is probably running.
    // The database we use can only be accessed from a single process. It will fail later if we have more than one running
    let lock_path = utils::enforced_system_root().join("ceridwen.lock");
    let lock_file_result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path);
    let lock_file = match lock_file_result {
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                println!("Already existing ceridwen lock file at {:?}. If the previous run crashed remove the file manually", &lock_path);
                return;
            } else {
                println!("Error obtaining lock file: {:?} {}", &lock_path, e);
                return;
            }
        }
        Ok(f) => f,
    };

    // Run the application and handle error responses
    let result = ceridwen_main().await;
    if result.is_err() {
        let err = result.unwrap_err();
        println!("Error running ceridwen: {}", err);
    }

    // Clean up the lock file
    drop(lock_file);
    // If this doesn't work we cant do much about it, but tell the user
    if let Err(e) = fs::remove_file(&lock_path) {
        println!("Could not delete lock file: {e}");
        println!("You will probably need to clean this up manually! {lock_path:?}");
    }
}

async fn ceridwen_main() -> Result<(), Error> {
    println!("Ceridwen starting. Loading config");
    let config = Config::load()?;

    println!("Config loaded. Setting up logging");
    configure_logging(&config)?;
    info!("Logging setup. Logging level set to {}", &config.log_level);

    // start up the web server
    let server = server::run_server(config)?;

    // start up the crawler
    // TODO: figure out scheduling of the crawler task so we don't have to restart the server to trigger this
    let crawler_task = tokio::spawn(crawler::crawler_main());

    // now we can await on the server and let it run
    server.await?;

    _ = crawler_task.await?;

    Ok(())
}

fn configure_logging(config: &Config) -> Result<Handle, Error> {
    let log_path = utils::system_root().join("logs");

    let log_level = LevelFilter::from_str(&config.log_level)?;

    let window_size = 5;
    let fixed_window_roller = FixedWindowRoller::builder().build(
        log_path.join("ceridwen_{}.log").to_str().unwrap(),
        window_size,
    )?;

    let size_limit = 50 * 1024 * 1024; // 50MB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);

    let compound_policy =
        CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));

    let file_logger = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .append(true)
        .build(log_path.join("ceridwen.log"), Box::new(compound_policy))?;

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
                .build(log_level),
        )?;

    let handle = log4rs::init_config(config)?;
    Ok(handle)
}
