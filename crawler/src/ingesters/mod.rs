use ceridwen::config::Config;
use ceridwen::config::Ingester;
use ceridwen::error::Error;
use chrono::Days;
use chrono::Utc;
use log::info;
use log::warn;

mod rss_ingester;

/// These are tools for reading in a data source and adding to the index so we can search things.
///

pub async fn process_ingester(ingester_config: Ingester, config: Config) {
    let name = ingester_config.name.clone();
    let result = process(ingester_config, config).await;
    if result.is_err() {
        warn!(
            "Error processing ingester {}: {}",
            name,
            result.unwrap_err()
        )
    }
}

async fn process(ingester_config: Ingester, mut config: Config) -> Result<(), Error> {
    let next_run = ingester_config
        .last_update
        .checked_add_days(Days::new(ingester_config.interval_days))
        .unwrap();

    let name = ingester_config.name.clone();

    if next_run > Utc::now() {
        info!(
            "Not ready to process {} until {}",
            ingester_config.name, next_run
        );
        return Ok(());
    }

    match ingester_config.ingester_type.as_str() {
        "rss" => rss_ingester::process_rss(ingester_config, config.clone()).await,
        "wikipedia" => process_wikipedia(ingester_config, config.clone()).await,
        "spider" => process_spider(ingester_config, config.clone()).await,
        a => Err(Error::UnknownIngester(a.to_string())),
    }?;

    // update the config so its got the right date on it.
    for ingester in config.targets.iter_mut() {
        if ingester.name == name {
            ingester.last_update = Utc::now();
            break;
        }
    }
    config.save()?;

    Ok(())
}

async fn process_wikipedia(_ingester_config: Ingester, _config: Config) -> Result<(), Error> {
    todo!();
}

async fn process_spider(_ingester_config: Ingester, _config: Config) -> Result<(), Error> {
    todo!();
}
