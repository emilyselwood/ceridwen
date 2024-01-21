use bytes::Buf;
use ceridwen::config::Config;
use ceridwen::config::Ingester;
use ceridwen::error::Error;
use ceridwen::index;
use ceridwen::index::Index;
use ceridwen::page::Page;
use chrono::Days;
use chrono::Utc;
use log::info;
use log::warn;
use rss::Channel;
use url::Url;

use crate::robots_text;
use crate::web_client;

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
        "rss" => process_rss(ingester_config, config.clone()).await,
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

async fn process_rss(ingester_config: Ingester, config: Config) -> Result<(), Error> {
    let base_url = match ingester_config.base_url {
        Some(u) => u,
        None => return Err(Error::MissingBaseUrl),
    };

    info!(
        "Processing RSS Feed! {} at {}",
        ingester_config.name, &base_url
    );

    let client = web_client::get_client(&config)?;

    let target_url = Url::parse(&base_url)?;

    if !robots_text::check_robots_file(&client, &target_url).await? {
        info!("Sites robot.txt disallows us to process it. Not indexing {base_url}");
        return Ok(());
    }
    info!("Allowed to index {base_url} by robots.txt");

    let rss_bytes = web_client::get_url(&client, &target_url).await?;
    let channel = Channel::read_from(rss_bytes.reader())?;

    let mut index = Index::load(&index::index_dir())?;

    for item in channel.items().iter() {
        info!(
            "found item {} with url: {}",
            item.title().unwrap_or("no title"),
            item.link().unwrap_or("No link")
        );

        if item.link().is_none() {
            info!(
                "skipping '{}' as it does not have a link",
                item.title().unwrap_or("No title")
            );
            continue;
        }

        let url = Url::parse(item.link().unwrap())?;
        // Create a page object
        let page = Page {
            url,
            title: item.title().unwrap_or("No title").to_string(),
            content: item
                .content()
                .unwrap_or(item.description().unwrap_or("no content"))
                .to_string(),
        };

        // add page to the index
        index.add_page(&page)?;
    }

    info!("Done processing rss feed {}", ingester_config.name);
    Ok(())
}

async fn process_wikipedia(_ingester_config: Ingester, _config: Config) -> Result<(), Error> {
    todo!();
}

async fn process_spider(_ingester_config: Ingester, _config: Config) -> Result<(), Error> {
    todo!();
}
