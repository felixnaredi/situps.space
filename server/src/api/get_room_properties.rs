use std::{collections::HashMap, marker::PhantomData};

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

#[derive(Debug, Serialize, Deserialize)]
struct Base64EncodedRequest<T> where T: Serialize {
    b64: String,
    _marker: PhantomData<T>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRoomPropertiesRequest
{
    room_id: u64,

    // #[serde(default = "default_as_empty_vec")]
    // dates: Vec<GregorianScheduleDate>,

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
    use hyper::{
        Client,
        Uri,
    };
    use tokio::sync::oneshot;

    use super::*;

    #[tokio::test]
    async fn requests_are_parsed_correctly()
    {
        let (tx_terminate_server, rx_terminate_server) = oneshot::channel();
        println!("channel created");

        let (_, server) = warp::serve(
            warp::path!("api" / "room" / "get-room-properties")
                .and(warp::query::<GetRoomPropertiesRequest>())
                .map(|request| {
                    println!("{:#?}", request);
                    warp::reply()
                }),
        )
        .bind_with_graceful_shutdown(([127, 0, 0, 1], 8000), async move {
            rx_terminate_server.await.ok();
        });

        let server = tokio::task::spawn(server);

        let request = GetRoomPropertiesRequest {
            room_id: 1,
            // dates: vec![
            //     GregorianScheduleDate::new(2023, 4, 29),
            //     GregorianScheduleDate::new(2023, 4, 30),
            //     GregorianScheduleDate::new(2023, 5, 1),
            // ],
            entries: false,
            users: false,
            display_name: false,
            url: false,
            broadcast: false,
        };
        println!("{}", serde_urlencoded::to_string(&request).unwrap());

        let client = Client::new();
        let res = client
            .get(Uri::from_static(
                "http://127.0.0.1:8000/api/room/get-room-properties?roomId=1&entries=true",
            ))
            .await
            .unwrap();
        println!("status: {}", res.status());
        println!("body: {:?}", res.body());

        tx_terminate_server.send(()).unwrap();
        server.await.ok();
    }
}
