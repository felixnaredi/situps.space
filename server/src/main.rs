mod schemes;

use std::{
    env,
    fs::File,
};

use mongodb::{
    bson::doc,
    options::{
        ClientOptions,
        ServerApi,
        ServerApiVersion,
    },
    Client,
};
use serde::{
    Deserialize,
    Serialize,
};
use simplelog::SimpleLogger;
use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct Config
{
    database_url: String,
    database: String,
    socket_address: [u8; 4],
    socket_port: u16,
}

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

    let mut client_options = ClientOptions::parse(config.database_url).await?;
    client_options.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

    let client = Client::with_options(client_options)?;
    let response = client
        .database("admin")
        .run_command(doc! { "ping": 1}, None)
        .await?;

    log::info!("successfully pinged database! - {}", response);

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run((config.socket_address, config.socket_port)).await;
    Ok(())
}
