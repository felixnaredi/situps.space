use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use warp::Filter;

use crate::schemes::{
    Entry,
    GregorianScheduleDate,
    User,
};

fn default_as_false() -> bool
{
    false
}

fn default_as_empty_vec<T>() -> Vec<T>
{
    Vec::new()
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
struct GetRoomPropertiesRequest
{
    room_id: u64,

    #[serde(default = "default_as_empty_vec")]
    dates: Vec<GregorianScheduleDate>,

    #[serde(default = "default_as_false")]
    entries: bool,

    #[serde(default = "default_as_false")]
    users: bool,

    #[serde(default = "default_as_false")]
    display_name: bool,

    #[serde(default = "default_as_false")]
    url: bool,

    #[serde(default = "default_as_false")]
    broadcast: bool,
}

#[derive(Debug, Serialize)]
struct GetRoomPropertiesResponse
{
    room_id: u64,
    entries: HashMap<GregorianScheduleDate, Entry>,
    users: HashMap<GregorianScheduleDate, User>,
    display_name: Option<String>,
    url: Option<String>,
    broadcast: Option<String>,
}

pub fn handle() -> impl warp::Filter
{
    warp::path!("api" / "room" / "get-room-properties")
        .and(warp::query::<GetRoomPropertiesRequest>())
        .map(|request| GetRoomPropertiesResponse {
            room_id: 1,
            entries: HashMap::new(),
            users: HashMap::new(),
            display_name: None,
            url: None,
            broadcast: None,
        })
}

#[cfg(test)]
mod test
{
    use std::{
        cell::RefCell,
        ops::DerefMut,
        str::FromStr,
        sync::Mutex,
    };

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
        Database,
    };
    use tokio::{
        sync::oneshot,
        task::JoinError,
    };

    use super::*;
    use crate::api::Base64EncodedRequest;

    fn server_expecting_request(
        port: u16,
        value: Base64EncodedRequest<GetRoomPropertiesRequest>,
    ) -> (
        tokio::sync::oneshot::Sender<()>,
        impl warp::Future<Output = Result<(), JoinError>> + 'static,
    )
    {
        let (tx, rx) = oneshot::channel();

        let (_, server) = warp::serve(
            warp::path!("api" / "room" / "get-room-properties")
                .and(warp::query::<Base64EncodedRequest<GetRoomPropertiesRequest>>())
                .map(move |request| {
                    println!("server received: {:?}", request);
                    assert_eq!(request, value);
                    warp::reply()
                }),
        )
        .bind_with_graceful_shutdown(([127, 0, 0, 1], port), async move {
            rx.await.ok();
        });

        (tx, tokio::spawn(server))
    }

    fn send_request(port: u32, url_parameters: &str) -> hyper::client::ResponseFuture
    {
        use hyper::{
            Client,
            Uri,
        };

        let client = Client::new();
        client.get(
            Uri::from_str(&format!(
                "http://127.0.0.1:{}/api/room/get-room-properties?{}",
                port, url_parameters
            ))
            .unwrap(),
        )
    }

    #[tokio::test]
    async fn parse_generated_requests()
    {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        for size in [(0..=0), (1..=1), (2..=4), (2..=4), (20..=40), (80..=160)] {
            //
            // Generate request.
            //
            let request = GetRoomPropertiesRequest {
                room_id: rng.gen(),
                dates: (0..rng.gen_range(size))
                    .into_iter()
                    .map(|_| {
                        GregorianScheduleDate::new(
                            rng.gen_range(1000..3000),
                            rng.gen_range(1..=12),
                            rng.gen_range(1..28),
                        )
                    })
                    .collect(),
                entries: rng.gen(),
                users: rng.gen(),
                display_name: rng.gen(),
                url: rng.gen(),
                broadcast: rng.gen(),
            };
            let request = Base64EncodedRequest(request);

            //
            // Launch server.
            //
            let (tx, server) = server_expecting_request(8000, request.clone());

            //
            // Send url encoded request to the server and check that it was successful.
            //
            let response = send_request(8000, &serde_urlencoded::to_string(&request).unwrap())
                .await
                .unwrap();
            assert!(response.status().is_success());

            //
            // Close down server.
            //
            tx.send(()).unwrap();
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn parse_request_with_only_room_id()
    {
        //
        // Launch server.
        //
        let (tx, server) = server_expecting_request(
            8001,
            Base64EncodedRequest(GetRoomPropertiesRequest {
                room_id: 17956551056096027663,
                dates: vec![],
                entries: false,
                users: false,
                display_name: false,
                url: false,
                broadcast: false,
            }),
        );

        //
        // Send url encoded request to the server and check that it was successful.
        //
        let response = send_request(8001, "b64=eyJyb29tSWQiOjE3OTU2NTUxMDU2MDk2MDI3NjYzfQo")
            .await
            .unwrap();
        assert!(response.status().is_success());

        //
        // Close down server.
        //
        tx.send(()).unwrap();
        server.await.unwrap();
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateUser
    {
        display_name: String,
        theme: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateRoom
    {
        display_name: String,
        url: String,
        broadcast: String,
        users: Vec<mongodb::bson::oid::ObjectId>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Id
    {
        date: GregorianScheduleDate,
        room: mongodb::bson::oid::ObjectId,
        user: mongodb::bson::oid::ObjectId,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateEntry
    {
        #[serde(rename = "_id")]
        id: Id,
        amount: i32,
    }

    static DATABASE_IS_FILLED: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

    async fn ids(
        db: &Database,
        collection: &str,
    ) -> Result<Vec<mongodb::bson::oid::ObjectId>, Box<dyn std::error::Error>>
    {
        #[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
        struct Query
        {
            _id: mongodb::bson::oid::ObjectId,
        }

        Ok(db
            .collection::<Query>(collection)
            .find(
                doc! {},
                Some(
                    FindOptions::builder()
                        .projection(doc! {"_id": true })
                        .build(),
                ),
            )
            .await?
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .map(|x| x._id)
            .collect())
    }

    async fn database() -> Result<Database, Box<dyn std::error::Error>>
    {
        let mut client_options = ClientOptions::parse("mongodb://127.0.0.1:27017").await?;
        client_options.server_api =
            Some(ServerApi::builder().version(ServerApiVersion::V1).build());

        let client = Client::with_options(client_options)?;
        let db = client.database("test-get-room-properties");

        match DATABASE_IS_FILLED.lock().unwrap().borrow_mut().deref_mut() {
            x if *x == false => {
                //
                // Drop old database.
                //
                db.drop(None).await.unwrap();

                //
                // Insert users.
                //
                db.collection("users")
                    .insert_many(
                        [
                            //
                            // display_name , theme
                            //
                            (".}1qyp}~L%", "forrest"),
                            ("*ErF.0 $=?ze", "forrest"),
                            ("7e{Fm18L|p", "ocean"),
                            ("up/|CThg", "ocean"),
                        ]
                        .into_iter()
                        .map(|(display_name, theme)| CreateUser {
                            display_name: display_name.to_owned(),
                            theme: theme.to_owned(),
                        }),
                        None,
                    )
                    .await?;
                let u = ids(&db, "users").await?;

                //
                // Insert rooms.
                //
                db.collection("rooms")
                    .insert_many(
                        [
                            //
                            // display_name    , users
                            //
                            ("OXtvty)RBVzmlvY-", vec![u[0], u[1], u[2], u[3]]),
                            ("m(%0~FiwluTS$", vec![u[0], u[2]]),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, (display_name, users))| CreateRoom {
                            display_name: display_name.to_owned(),
                            url: format!("http://127.0.0.1:8002/room/{}", i),
                            broadcast: format!("http://127.0.0.1:8002/room/broadcast/{}", i),
                            users,
                        }),
                        None,
                    )
                    .await?;
                let r = ids(&db, "rooms").await?;

                //
                // Insert entries.
                //
                db.collection("entries")
                    .insert_many(
                        [
                            //
                            // date                                 , room, user, amount
                            //
                            (GregorianScheduleDate::new(1555, 2, 13), r[0], u[0], 10),
                            (GregorianScheduleDate::new(1555, 2, 13), r[0], u[1], 11),
                            (GregorianScheduleDate::new(1555, 2, 13), r[0], u[2], 12),
                            (GregorianScheduleDate::new(1555, 2, 13), r[0], u[3], 13),
                            (GregorianScheduleDate::new(1555, 2, 14), r[0], u[1], 21),
                            (GregorianScheduleDate::new(1555, 2, 14), r[0], u[2], 22),
                            (GregorianScheduleDate::new(1555, 2, 14), r[0], u[3], 23),
                            (GregorianScheduleDate::new(1555, 2, 16), r[0], u[0], 30),
                            (GregorianScheduleDate::new(1555, 2, 16), r[0], u[1], 31),
                            (GregorianScheduleDate::new(1555, 2, 17), r[0], u[0], 40),
                            (GregorianScheduleDate::new(1555, 2, 13), r[1], u[0], 110),
                            (GregorianScheduleDate::new(1555, 2, 17), r[1], u[2], 120),
                        ]
                        .map(|(date, room, user, amount)| CreateEntry {
                            id: Id { date, room, user },
                            amount,
                        }),
                        None,
                    )
                    .await?;

                *x = true;
            }
            _ => {}
        }

        Ok(db)
    }

    #[tokio::test]
    async fn f()
    {
        database().await.unwrap();
    }

    #[tokio::test]
    async fn g()
    {
        database().await.unwrap();
    }
}
