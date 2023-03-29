mod api;
mod schemes;
mod request;

use std::{
    env,
    fs::File,
};

use simplelog::SimpleLogger;

use crate::api::{
    Config,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    SimpleLogger::init(log::LevelFilter::Debug, simplelog::Config::default())?;
    log::info!("logger initialized");

    //
    // Load config.
    //
    let config_file_name = format!(
        "config.{}.local.json",
        &env::var("SERVER_MODE").unwrap_or("development".into())
    );
    let config: Config = serde_json::from_reader(File::open(&config_file_name).expect(&format!(
        "expected to open config file '{}'",
        config_file_name
    )))?;

    //
    // Run API.
    //
    api::serve(&config).await?;

    Ok(())
}
