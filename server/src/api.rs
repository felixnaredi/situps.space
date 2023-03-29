use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};

use futures::{
    SinkExt,
    StreamExt,
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
};

use crate::{
    db,
    entry_event::{
        self,
        Request,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config
{
    database_url: String,
    database: String,
    socket_address: [u8; 4],
    socket_port: u16,
}

type ConnectedClients =
    Arc<RwLock<HashMap<SocketAddr, mpsc::UnboundedSender<entry_event::Broadcast>>>>;

///
/// Launches an API.
///
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

///
/// Initializese the database.
///
/// ## NOTE
/// The database is not a singleton. Each call to this function established a new client.
///
async fn initialize_database(config: &Config) -> Result<Client, Box<dyn std::error::Error>>
{
    let mut client_options = ClientOptions::parse(&config.database_url).await?;
    client_options.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

    Ok(Client::with_options(client_options)?)
}

///
/// Sets up all required connections needed to handle a /entry WebSocket.
///
async fn handle_entry_websocket(
    websocket: WebSocket,
    addr: SocketAddr,
    db: Arc<Database>,
    connected_clients: ConnectedClients,
)
{
    use entry_event::Response;

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

        tokio::task::spawn(async move {
            use entry_event::Broadcast;

            let mut rx = UnboundedReceiverStream::new(rx);

            //
            // Listen to broadcast events.
            //
            while let Some(request) = rx.next().await {
                log::debug!("{:?} - parsed request: {:?}", addr, request);

                match request {
                    //
                    // Respond to broadcast `Update` event.
                    //
                    Broadcast::UpdateEntry(entry) => {
                        user_ws_tx
                            .write()
                            .await
                            .send(Response::update_entry(entry.clone()))
                            .await
                            .unwrap();
                    }
                }
            }
        });
    };

    //
    // Receive request from the newly connected client.
    //
    while let Some(result) = user_ws_rx.next().await {
        log::debug!("received request: {:?}", result);

        let request = match parse_raw_request(result) {
            Ok(request) => request,
            Err(error) => {
                log::error!("websocket error: addr: {:?} - {:?}", addr, error);
                break;
            }
        };

        log::debug!("processing request {:?}", request);

        use entry_event::Broadcast;
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
                    .send(Response::get_entry_data(data.entry_key, entry_data))
                    .await
                    .unwrap();
            }

            //j
            // Respond to clients `Update` request.
            //
            UpdateEntry(data) => {
                db::update_entry(&db, &data.entry).await.unwrap();
                notify_connected_clients(&connected_clients, Broadcast::UpdateEntry(data.entry))
                    .await
            }
        }
    }

    log::info!("{:?} disconnected", addr);
    connected_clients.write().await.remove(&addr);
}

///
/// A raw request might not even have a `&str` in its message.
///
fn parse_raw_request<E: std::error::Error + 'static>(
    msg: Result<Message, E>,
) -> Result<Request, Box<dyn std::error::Error>>
{
    parse_request(msg?.to_str().map_err(|e| format!("{:?}", e))?)
}

///
/// Parse request.
///
fn parse_request(msg: &str) -> Result<Request, Box<dyn std::error::Error>>
{
    Ok(serde_json::from_str(msg)?)
}

///
/// Update all connected clients of the event.
///
async fn notify_connected_clients(clients: &ConnectedClients, msg: entry_event::Broadcast)
{
    for tx in clients.read().await.values() {
        tx.send(msg.clone()).unwrap();
    }
}
