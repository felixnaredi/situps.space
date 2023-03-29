use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};

use futures::{
    channel::mpsc::UnboundedReceiver,
    Sink,
    SinkExt,
    StreamExt,
    TryStreamExt,
};
use mongodb::{
    bson::doc,
    options::{
        ClientOptions,
        FindOptions,
        ServerApi,
        ServerApiVersion,
    },
    Client,
    Database,
};
use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::{
    mpsc,
    RwLock,
};
use warp::{
    ws::{
        Message,
        WebSocket,
    },
    Filter,
    Future,
};

use crate::{
    api,
    schemes::User,
};

use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config
{
    database_url: String,
    database: String,
    socket_address: [u8; 4],
    socket_port: u16,
}

type ConnectedUsers = Arc<RwLock<HashMap<SocketAddr, mpsc::UnboundedSender<Message>>>>;

pub async fn serve(config: &Config) -> Result<(), Box<dyn std::error::Error>>
{
    let client = initialize_database(config).await?;
    let response = client
        .database("admin")
        .run_command(doc! { "ping": 1}, None)
        .await?;
    log::debug!("successfully pinged database! - {}", response);

    let db = Arc::new(client.database(&config.database));
    log::debug!("using database {}", &config.database);

    let log_request = warp::addr::remote()
        .and(warp::path::full())
        .map(|addr, path| {
            if let Some(addr) = addr {
                log::info!("request from {:?} - {:?}", addr, path);
            } else {
                log::info!("request from NONE - {:?}", path);
            }
        });

    let api_users = warp::path!("api" / "users").and(log_request).and_then({
        let db = db.clone();
        move |()| {
            let db = db.clone();
            async move {
                match get_users_from_db(&db).await {
                    Ok(users) => Ok(warp::reply::json(&users)),
                    Err(error) => {
                        log::error!("{}", error);
                        Err(warp::reject())
                    }
                }
            }
        }
    });

    let connected_users = Arc::new(RwLock::new(HashMap::new()));

    let socket_entry = warp::path("entry")
        .and(log_request)
        .and(warp::ws())
        .and(warp::addr::remote())
        .map({
            let db = db.clone();
            let connected_users = connected_users.clone();
            move |_, ws: warp::ws::Ws, addr: Option<_>| {
                let db = db.clone();
                let connected_users = connected_users.clone();
                ws.on_upgrade(move |websocket| async move {
                    handle_entry_websocket(websocket, addr.unwrap(), &db, connected_users).await
                })
            }
        });

    Ok(warp::serve(api_users.or(socket_entry))
        .run((config.socket_address, config.socket_port))
        .await)
}

async fn initialize_database(config: &Config) -> Result<Client, Box<dyn std::error::Error>>
{
    let mut client_options = ClientOptions::parse(&config.database_url).await?;
    client_options.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

    Ok(Client::with_options(client_options)?)
}

async fn get_users_from_db(db: &Database) -> Result<Vec<User>, Box<dyn std::error::Error>>
{
    let cursor = db
        .collection::<User>("users")
        .find(
            doc! {},
            FindOptions::builder()
                .projection(doc! {
                    "_id": "$_id", "displayName": 1, "theme": 1
                })
                .build(),
        )
        .await?;
    Ok(cursor.try_collect::<Vec<_>>().await?)
}

async fn handle_entry_websocket(
    websocket: WebSocket,
    addr: SocketAddr,
    db: &Database,
    connected_users: ConnectedUsers,
)
{
    let (mut user_ws_tx, mut user_ws_rx) = websocket.split();

    //
    // Create user id and add receiving channel to `connected_users`.
    //
    let (tx, rx) = mpsc::unbounded_channel();
    connected_users.write().await.insert(addr, tx);


    let mut rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(async move {
        user_ws_tx
            .send(Message::text("established connection with server"))
            .await
            .unwrap();
        while let Some(message) = rx.next().await {
            log::debug!("{:?} - received message: {:?}", addr, message);
            user_ws_tx.send(Message::text("update")).await.unwrap();
        }
    });

    while let Some(message) = user_ws_rx.next().await {
        let message = message.unwrap();
        for tx in connected_users.read().await.values() {
            tx.send(message.clone()).unwrap();
        }
    }

    
}
