use std::sync::Arc;

use futures::TryStreamExt;
use mongodb::{
    bson::doc,
    options::{
        ClientOptions,
        FindOptions,
        ServerApi,
        ServerApiVersion,
    },
    Client,
};
use serde::{
    Deserialize,
    Serialize,
};
use warp::{
    Filter,
    Future,
};

use crate::schemes::User;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config
{
    database_url: String,
    database: String,
    socket_address: [u8; 4],
    socket_port: u16,
}

pub struct API
{
    config: Config,
    client: Client,
}

impl API
{
    /// Initializes the API and launches the database.
    pub async fn new(config: Config) -> Result<API, Box<dyn std::error::Error>>
    {
        let mut client_options = ClientOptions::parse(&config.database_url).await?;
        client_options.server_api =
            Some(ServerApi::builder().version(ServerApiVersion::V1).build());

        let client = Client::with_options(client_options)?;
        let response = client
            .database("admin")
            .run_command(doc! { "ping": 1}, None)
            .await?;

        log::info!("successfully pinged database! - {}", response);

        Ok(API { config, client })
    }

    /// Serves the API.
    pub fn run(&self) -> impl Future
    {
        let db = Arc::new(self.client.database(&self.config.database));

        // -----------------------------------------------------------------------------------------
        // Fetch users.
        // -----------------------------------------------------------------------------------------

        let users = {
            let db = db.clone();

            warp::path!("api" / "users").and_then(move || {
                let db = db.clone();
                async move {
                    match db
                        .collection::<User>("users")
                        .find(
                            doc! {},
                            FindOptions::builder()
                                .projection(doc! {
                                    "userID": "$_id", "displayName": 1, "theme": 1
                                })
                                .build(),
                        )
                        .await
                    {
                        Ok(cursor) => match cursor.try_collect::<Vec<_>>().await {
                            Ok(data) => Ok(warp::reply::json(&data)),
                            Err(error) => {
                                log::error!("{}", error);
                                Err(warp::reject::not_found())
                            }
                        },
                        Err(error) => {
                            log::error!("{}", error);
                            Err(warp::reject::not_found())
                        }
                    }
                }
            })
        };

        // -----------------------------------------------------------------------------------------
        // Serve API.
        // -----------------------------------------------------------------------------------------

        let hello = warp::path!("hello" / String)
            .map(|name| format!("Hello, {}!", name))
            .or(users);
        warp::serve(hello).run((self.config.socket_address, self.config.socket_port))
    }
}
