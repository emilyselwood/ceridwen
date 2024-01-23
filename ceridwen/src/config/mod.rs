use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use time;
use toml;

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub targets: Vec<Ingester>,
    pub server: Server,
    pub crawler: Crawler,
    pub last_update: time::OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ingester {
    pub name: String,
    pub ingester_type: String,
    pub update_interval: time::Duration,
    pub base_url: Option<String>,
    pub last_update: time::OffsetDateTime,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    /// Log level for the server
    pub log_level: String,

    /// network port to use for the server
    pub port: u16,

    /// number of server workers needed. The default of 2 should be more than enough for most households.
    pub workers: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Crawler {
    /// Logging level to use when crawling
    pub log_level: String,

    /// number of worker processes to have when crawling. Depending on the number of targets you have you may need to
    /// increase this
    pub workers: usize,

    /// Minimum amount of time before the crawler will go back to a page to check for changes.
    pub min_update_interval: time::Duration,
}

impl Config {
    pub fn config_path() -> String {
        crate::system_root()
            .join("config.toml")
            .to_str()
            .unwrap()
            .to_string()
    }

    /// Create a default config object. Very rarely needed, shouldn't be used outside of init.
    pub fn new() -> Config {
        Config {
            targets: vec![
                // A test ingester for an rss feed with out a robots.txt file
                Ingester {
                    name: "parsecsreach".to_string(),
                    ingester_type: "rss".to_string(),
                    update_interval: time::Duration::days(7),
                    base_url: Some("https://parsecsreach.org/index.xml".to_string()),
                    last_update: time::OffsetDateTime::now_utc() - time::Duration::days(7),
                    options: HashMap::new(),
                },
                // A test ingester for an rss feed that has a robots.txt file
                // Ingester {
                //     name: "slate".to_string(),
                //     ingester_type: "rss".to_string(),
                //     interval_days: 1,
                //     base_url: Some("https://slate.com/feeds/all.rss".to_string()),
                //     last_update: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
                //     options: HashMap::new(),
                // },
            ],
            server: Server {
                log_level: "info".to_string(),
                port: 8080,
                workers: 2,
            },
            crawler: Crawler {
                log_level: "debug".to_string(),
                workers: 8,
                min_update_interval: time::Duration::days(1),
            },
            last_update: time::OffsetDateTime::now_utc(),
        }
    }

    /// Load the config from the standard place on disk.
    pub fn load() -> Result<Config, Error> {
        let config_path = Config::config_path();
        let file_content = match fs::read_to_string(config_path.clone()) {
            Ok(fc) => fc,
            Err(err) => {
                // Can't log here as we need the config to set up the loggers so it very likely won't work.
                println!(
                    "Could not open config file at {}: {}",
                    config_path.clone(),
                    err
                );
                println!("You may need to run 'ceridwen-init'.");
                return Err(Error::from(err));
            }
        };
        let config: Config = toml::from_str(file_content.as_str())?;

        Ok(config)
    }

    /// Save the this config object to a file in toml format
    pub fn save(&self) -> Result<(), Error> {
        let config_path = Config::config_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;

        Ok(())
    }
}
