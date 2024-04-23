use std::fs;
use std::path::Path;

use ceridwen::config;
use ceridwen::error::Error;
use ceridwen::index_sled;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// set up ceridwen
    ///
    /// This will create the needed folders, config, and database files
    Init {
        /// overwrite existing files if they exist
        #[clap(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    let result = match args.cmd {
        SubCommands::Init { force } => init_command(force).await,
    };

    if let Err(e) = result {
        println!("Error running admin command: {:?}", e);
    }
}

async fn init_command(force: bool) -> Result<(), Error> {
    let root_path = ceridwen::system_root();
    // make sure the rood dir exits
    if !root_path.exists() {
        println!("Creating root path {:?}", root_path);
        let result = fs::create_dir_all(&root_path);
        if result.is_err() {
            println!(
                "Could not create ceridwen directory at {:?}: {}",
                root_path,
                result.unwrap_err(),
            );
            return Err(Error::IndexDirDoesNotExist(
                root_path.to_str().unwrap().to_string(),
            ));
        }
    }

    // Write the config file
    let config_path = config::Config::config_path();
    if !Path::new(&config_path).exists() || force {
        println!("Creating base config at {config_path}");

        let new_config = config::Config::default();

        // TODO: ask about any user configurable settings

        new_config.save()?;
    } else {
        println!("Config already exists! Refusing to overwrite it");
    }

    // create the database
    let database_path = index_sled::index_path();
    if database_path.exists() {
        if force {
            println!("Deleting existing database {:?}", database_path);
            fs::remove_dir_all(index_sled::index_path())?;
        } else {
            println!("Database already exists. Refusing to overwrite existing database");
            return Err(Error::IndexDirAlreadyExists(
                database_path.to_str().unwrap().to_string(),
            ));
        }
    }

    println!("Creating new database");
    _ = index_sled::Index::load().await?;

    println!(
        "Setup complete. Please review the configuration file at {} and then run ceridwen-crawler to populate your index",
        config_path
    );
    Ok(())
}
