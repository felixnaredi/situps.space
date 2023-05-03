use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;
use serde::{
    Deserialize,
    Serialize,
};
use warp::Filter;

use super::Base64EncodedRequest;
use crate::schemes::{
    Entry,
    GregorianScheduleDate,
    User,
};

// -------------------------------------------------------------------------------------------------
// Crate public interface.
// -------------------------------------------------------------------------------------------------

/// The routes that handle a 'get-room-properties' request.
pub fn routes() -> impl Filter<Extract = (warp::reply::Json,), Error = warp::Rejection> + Clone
{
    warp::path!("api" / "room" / "get-room-properties").and(warp::query().map(
        |request: Base64EncodedRequest<GetRoomPropertiesRequest>| {
            // panic!();
            // warp::reply()
            warp::reply::json(&GetRoomPropertiesResponse {
                room_id: request.0.room_id,
                entries: None,
                users: None,
                display_name: None,
                url: None,
                broadcast: None,
            })
        },
    ))
}

// -------------------------------------------------------------------------------------------------
// Internal.
// -------------------------------------------------------------------------------------------------

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
    room_id: ObjectId,

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GetRoomPropertiesResponse
{
    room_id: ObjectId,
    entries: Option<HashMap<GregorianScheduleDate, Vec<Entry>>>,
    users: Option<HashMap<GregorianScheduleDate, Vec<User>>>,
    display_name: Option<String>,
    url: Option<String>,
    broadcast: Option<String>,
}

// -------------------------------------------------------------------------------------------------
// Tests.
// -------------------------------------------------------------------------------------------------

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
    use hyper::body::HttpBody;
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
                room_id: ObjectId::new(),
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
                room_id: ObjectId::from_bytes([62, 101, 67, 0, 114, 70, 127, 42, 108, 120, 49, 27]),
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
        let response = send_request(
            8001,
            "b64=eyJyb29tSWQiOnsiJG9pZCI6IjNlNjU0MzAwNzI0NjdmMmE2Yzc4MzExYiJ9fQo",
        )
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
        users: Vec<ObjectId>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Id
    {
        date: GregorianScheduleDate,
        room: ObjectId,
        user: ObjectId,
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
    ) -> Result<Vec<ObjectId>, Box<dyn std::error::Error>>
    {
        #[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
        struct Query
        {
            _id: ObjectId,
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
                            url: format!("https://test.situps.space/room/{}", i),
                            broadcast: format!("wss://test.situps.space/room/broadcast/{}", i),
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
    async fn request_and_check_response()
    {
        //
        // Fetch database and room ids.
        //
        let db = database().await.unwrap();
        let r = ids(&db, "rooms").await.unwrap();

        //
        // Launch the server.
        //
        let (tx, rx) = oneshot::channel();
        let (_, server) =
            warp::serve(routes()).bind_with_graceful_shutdown(([127, 0, 0, 1], 8003), async move {
                rx.await.ok();
            });
        let server = tokio::task::spawn(server);

        //
        // Either expect an exact value of the response or a predicate.
        //
        enum Expect
        {
            Value(GetRoomPropertiesResponse),
            Predicate(Box<dyn Fn(GetRoomPropertiesResponse) -> bool>),
        }

        //
        // Iterate over the request and expexted pairs.
        //
        for (request, expected) in [
            //
            // Empty request.
            //
            (
                GetRoomPropertiesRequest {
                    room_id: r[0].clone(),
                    dates: vec![],
                    entries: false,
                    users: false,
                    display_name: false,
                    url: false,
                    broadcast: false,
                },
                Expect::Value(GetRoomPropertiesResponse {
                    room_id: r[0].clone(),
                    entries: None,
                    users: None,
                    display_name: None,
                    url: None,
                    broadcast: None,
                }),
            ),
            //
            // `display_name`, `url` and `broadcast` are correct for room 0.
            //
            (
                GetRoomPropertiesRequest {
                    room_id: r[0].clone(),
                    dates: vec![],
                    entries: false,
                    users: false,
                    display_name: true,
                    url: true,
                    broadcast: true,
                },
                Expect::Value(GetRoomPropertiesResponse {
                    room_id: r[0].clone(),
                    entries: None,
                    users: None,
                    display_name: Some("OXtvty)RBVzmlvY-".to_owned()),
                    url: Some("https://test.situps.space/room/0".to_owned()),
                    broadcast: Some("wss://test.situps.space/room/broadcast/0".to_owned()),
                }),
            ),
            //
            // `display_name`, `url` and `broadcast` are correct for room 1.
            //
            (
                GetRoomPropertiesRequest {
                    room_id: r[1].clone(),
                    dates: vec![],
                    entries: false,
                    users: false,
                    display_name: true,
                    url: true,
                    broadcast: true,
                },
                Expect::Value(GetRoomPropertiesResponse {
                    room_id: r[1].clone(),
                    entries: None,
                    users: None,
                    display_name: Some("m(%0~FiwluTS$".to_owned()),
                    url: Some("https://test.situps.space/room/1".to_owned()),
                    broadcast: Some("wss://test.situps.space/room/broadcast/1".to_owned()),
                }),
            ),
            //
            // Request mask works properly.
            //
            (
                GetRoomPropertiesRequest {
                    room_id: r[1].clone(),
                    dates: vec![],
                    entries: true,
                    users: false,
                    display_name: true,
                    url: false,
                    broadcast: true,
                },
                Expect::Predicate(Box::new(|response| {
                    matches!(response.entries, Some(_))
                        && matches!(response.users, None)
                        && matches!(response.display_name, Some(_))
                        && matches!(response.url, None)
                        && matches!(response.broadcast, Some(_))
                })),
            ),
            //
            // Check properties from the whole period for room 0.
            //
            (
                GetRoomPropertiesRequest {
                    room_id: r[0].clone(),
                    dates: vec![
                        GregorianScheduleDate::new(1555, 2, 13),
                        GregorianScheduleDate::new(1555, 2, 14),
                        GregorianScheduleDate::new(1555, 2, 14),
                        GregorianScheduleDate::new(1555, 2, 16),
                        GregorianScheduleDate::new(1555, 2, 17),
                    ],
                    entries: true,
                    users: true,
                    display_name: false,
                    url: false,
                    broadcast: false,
                },
                Expect::Predicate(Box::new(|response| {
                    response.display_name == None
                        && response.url == None
                        && response.broadcast == None
                        && response.entries.unwrap().into_values().flatten().count() == 10
                        && response
                            .users
                            .as_ref()
                            .unwrap()
                            .get(&GregorianScheduleDate::new(1555, 2, 13))
                            .unwrap()
                            .len()
                            == 4
                        && response
                            .users
                            .unwrap()
                            .get(&GregorianScheduleDate::new(1555, 2, 14))
                            .unwrap()
                            .len()
                            == 3
                })),
            ),
        ] {
            let response = send_request(
                8003,
                &serde_urlencoded::to_string(Base64EncodedRequest(request)).unwrap(),
            )
            .await
            .unwrap();

            assert!(response.status().is_success());

            let response = serde_json::from_slice::<GetRoomPropertiesResponse>(
                &response.into_body().data().await.unwrap().unwrap(),
            )
            .unwrap();

            match expected {
                Expect::Value(expected) => assert_eq!(expected, response),
                Expect::Predicate(p) => assert!(p(response)),
            };
        }

        tx.send(()).unwrap();
        server.await.unwrap();
    }
}
