use ceridwen::config::Config;
use ceridwen::error::Error;
use env_logger::Env;
use log::info;

mod ingesters;
mod robots_text;
mod web_client;

#[tokio::main]
async fn main() {
    let result = crawler_main().await;
    if result.is_err() {
        let err = result.unwrap_err();
        print!("Error running crawler: {}", err);
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

    Ok(())
}
