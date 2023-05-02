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
    use std::str::FromStr;

    use hyper::{
        Client,
        Uri,
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
}
