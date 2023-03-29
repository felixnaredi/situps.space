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
    entry_event::{
        self,
        request,
        Request, Response,
    },
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

type ConnectedUsers = Arc<RwLock<HashMap<SocketAddr, mpsc::UnboundedSender<entry_event::Request>>>>;

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
    connected_clients: ConnectedUsers,
)
{
    let (user_ws_tx, mut user_ws_rx) = websocket.split();

    let user_ws_tx = Arc::new(RwLock::new(user_ws_tx));

    user_ws_tx
        .write()
        .await
        .send(Response::connection_established())
        .await
        .unwrap();

    //
    // Create broadcast channel for the connected client. Add it to the hash map of connected
    // clients.
    //
    let (tx, rx) = mpsc::unbounded_channel();
    connected_clients.write().await.insert(addr, tx);

    //
    // Spawn thread for the broadcast channel listener.
    //
    {
        let user_ws_tx = user_ws_tx.clone();

        tokio::task::spawn({
            let db = db.clone();
            async move {
                let mut rx = UnboundedReceiverStream::new(rx);

                while let Some(request) = rx.next().await {
                    log::debug!("{:?} - parsed request: {:?}", addr, request);

                    match request {
                        //
                        // Respond to another clients `GetEntryData` request.
                        //
                        entry_event::Request::GetEntryData(_) => {}

                        //
                        // Respond to another clients `Update` request.
                        //
                        entry_event::Request::Update() => {
                            user_ws_tx
                                .write()
                                .await
                                .send(Message::text(
                                    serde_json::to_string(&entry_event::Response::Update())
                                        .unwrap(),
                                ))
                                .await
                                .unwrap();
                        }
                    }
                }
            }
        });
    };

    //
    // Receive messages from the newly connected client.
    //
    while let Some(result) = user_ws_rx.next().await {
        let request = match result.map(parse_request) {
            Ok(request) => request,
            Err(error) => {
                log::error!("websocket error: addr: {:?} - {:?}", addr, error);
                break;
            }
        };

        log::debug!("processing request {:?}", request);

        use entry_event::Request::*;

        //
        // Request handler.
        //
        match request {
            //
            // Respond to clients `GetEntryData` request.
            //
            GetEntryData(data) => {
                let entry_data = db::get_entry_data(&db, &data.entry_key).await.unwrap();
                log::debug!("entry_data: {:?}", entry_data);
                user_ws_tx
                    .write()
                    .await
                    .send(entry_event::Response::get_entry_data(entry_data))
                    .await
                    .unwrap();
            }

            //
            // Respond to clients `Update` request.
            //
            Update() => notify_connected_clients(&connected_clients, request).await,
        }
    }

    log::info!("{:?} disconnected", addr);
    connected_clients.write().await.remove(&addr);
}

///
/// Parse request.
///
fn parse_request(msg: Message) -> Request
{
    // TODO:
    //   `msg.to_str().unwrap()` should be handled better.
    serde_json::from_str(msg.to_str().unwrap()).unwrap()
}

///
/// Update all connected clients of the event.
///
async fn notify_connected_clients(clients: &ConnectedUsers, msg: entry_event::Request)
{
    for tx in clients.read().await.values() {
        tx.send(msg.clone()).unwrap();
    }
}
