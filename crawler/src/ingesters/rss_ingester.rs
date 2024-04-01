use crate::error::Error;
use bytes::Buf;
use ceridwen::config::Config;
use ceridwen::config::Ingester;
use ceridwen::index;
use ceridwen::index::Index;
use ceridwen::page::Page;
use log::info;
use rss::Channel;
use url::Url;

use crate::robots_text;
use crate::web_client;

pub(crate) async fn process_rss(ingester_config: Ingester, config: Config) -> Result<(), Error> {
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
        index
            .add_page(&page, ingester_config.update_interval)
            .await?;
    }

    info!("Done processing rss feed {}", ingester_config.name);
    Ok(())
}
