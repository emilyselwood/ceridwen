/// These are tools for reading in a data source and adding to the index so we can search things.
use crate::config::Config;
use crate::config::Ingester;
use crate::error::Error;
use crate::index::postgres::PostgresIndex;
use crate::index::Index;
use chrono::Utc;
use log::info;
use log::warn;

mod rss_ingester;
mod wikipedia;

/// entry point and error logging wrapper
pub async fn process_ingester(ingester_config: Ingester, config: Config) {
    let name = ingester_config.name.clone();
    let result = process(ingester_config, config).await;
    if let Err(e) = result {
        warn!("Error processing ingester {}: {}", name, e)
    }
}

/// actual main processing function.
async fn process(ingester_config: Ingester, mut config: Config) -> Result<(), Error> {
    let mut index = PostgresIndex::connect(&config)?;

    let next_run = ingester_config.last_update + ingester_config.update_interval;

    let name = ingester_config.name.clone();

    if next_run > Utc::now() {
        info!(
            "Not ready to process {} until {}",
            ingester_config.name, next_run
        );
        return Ok(());
    }

    let start_time = Utc::now();

    match ingester_config.ingester_type.as_str() {
        "rss" => rss_ingester::process_rss(ingester_config, config.clone(), &mut index).await,
        "wikipedia" => {
            wikipedia::process_wikipedia(ingester_config, config.clone(), &mut index).await
        }
        "spider" => process_spider(ingester_config, config.clone(), &mut index).await,
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

    let duration = Utc::now() - start_time;
    info!("Processing {} took {:?}", &name, duration);

    Ok(())
}

async fn process_spider<A, T: Index<A>>(
    _ingester_config: Ingester,
    _config: Config,
    _index: &mut T,
) -> Result<(), Error> {
    todo!();
}
