use crate::error::Error;
use ceridwen::config::Config;
use ceridwen::config::Ingester;
use log::info;
use log::warn;

mod rss_ingester;
mod wikipedia;

/// These are tools for reading in a data source and adding to the index so we can search things.
///

/// entry point and error logging wrapper
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

/// actual main processing function.
async fn process(ingester_config: Ingester, mut config: Config) -> Result<(), Error> {
    let next_run = ingester_config.last_update + ingester_config.update_interval;

    let name = ingester_config.name.clone();

    if next_run > time::OffsetDateTime::now_utc() {
        info!(
            "Not ready to process {} until {}",
            ingester_config.name, next_run
        );
        return Ok(());
    }

    let start_time = time::Instant::now();

    match ingester_config.ingester_type.as_str() {
        "rss" => rss_ingester::process_rss(ingester_config, config.clone()).await,
        "wikipedia" => wikipedia::process_wikipedia(ingester_config, config.clone()).await,
        "spider" => process_spider(ingester_config, config.clone()).await,
        a => Err(Error::UnknownIngester(a.to_string())),
    }?;

    // update the config so its got the right date on it.
    for ingester in config.targets.iter_mut() {
        if ingester.name == name {
            ingester.last_update = time::OffsetDateTime::now_utc();
            break;
        }
    }
    config.save()?;

    let duration = start_time.elapsed();
    info!("Processing {} took {}", &name, duration);

    Ok(())
}

async fn process_spider(_ingester_config: Ingester, _config: Config) -> Result<(), Error> {
    todo!();
}
