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
        ServerApi,
        ServerApiVersion,
    },
    Client,
    Database,
};
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::{
    mpsc,
    RwLock,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{
        Message,
        WebSocket,
    },
    Filter,
    Future,
};

use crate::{
    db,
    entry_event,
    schemes::User,
};

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
                match db::get_users(&db).await {
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
                    handle_entry_websocket(websocket, addr.unwrap(), db.clone(), connected_users)
                        .await
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

async fn handle_entry_websocket(
    websocket: WebSocket,
    addr: SocketAddr,
    db: Arc<Database>,
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
    tokio::task::spawn({
        let db = db.clone();
        async move {
            user_ws_tx
                .send(Message::text("established connection with server"))
                .await
                .unwrap();
            while let Some(message) = rx.next().await {
                log::debug!("{:?} - received message: {:?}", addr, message);

                let request: entry_event::Request =
                    serde_json::from_str(message.to_str().unwrap()).unwrap();
                log::debug!("{:?} - parsed request: {:?}", addr, request);

                match request {
                    entry_event::Request::Get(data) => {
                        let entry_data = db::get_entry_data(&db, &data.entry_key).await.unwrap();
                        log::debug!("entry_data: {:?}", entry_data);
                        user_ws_tx
                            .send(Message::text(
                                serde_json::to_string(&entry_event::Response::get(entry_data))
                                    .unwrap(),
                            ))
                            .await
                            .unwrap();
                    }
                    entry_event::Request::Update() => {
                        user_ws_tx
                            .send(Message::text(
                                serde_json::to_string(&entry_event::Response::Update()).unwrap(),
                            ))
                            .await
                            .unwrap();
                    }
                }
            }
        }
    });

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(error) => {
                log::error!("websocket error: addr: {:?} - {:?}", addr, error);
                break;
            }
        };
        for tx in connected_users.read().await.values() {
            tx.send(msg.clone()).unwrap();
        }
    }

    log::info!("{:?} disconnected", addr);
    connected_users.write().await.remove(&addr);
}
