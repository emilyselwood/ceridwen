use chrono::Utc;
use log::info;

use crate::config::Config;
use crate::error::Error;

pub mod ingesters;
pub mod robots_text;
pub mod web_client;

pub async fn crawler_main() -> Result<(), Error> {
    info!("Crawler starting. Loading config");
    let config = Config::load()?;

    let process_start = Utc::now();

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

    let process_end = Utc::now() - process_start;
    info!("processing took: {:?}", process_end);

    Ok(())
}
