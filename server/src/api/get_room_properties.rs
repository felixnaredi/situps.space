use std::{
    collections::HashMap,
    marker::PhantomData,
};

use base64::Engine as _;
use serde::{
    de::Visitor,
    ser::SerializeMap,
    Deserialize,
    Serialize,
};
use warp::Filter;

use crate::{
    api::Base64EncodedRequest,
    schemes::{
        Entry,
        GregorianScheduleDate,
        User,
    },
};

fn default_as_false() -> bool
{
    false
}

fn default_as_empty_vec<T>() -> Vec<T>
{
    Vec::new()
}

// -------------------------------------------------------------------------------------------------
// TODO:
//   Hopefully the deserializer can be generalized for any reasonable T. It would fit good in its
//   own file.
//

// End of deserializer
// --------------------------------------------------------------------------------------------------

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
    use hyper::{
        Client,
        Uri,
    };
    use tokio::{
        sync::oneshot,
        task::JoinError,
    };

    use super::*;

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
                    println!("server received request: {:?}", request);
                    assert_eq!(request, value);
                    warp::reply()
                }),
        )
        .bind_with_graceful_shutdown(([127, 0, 0, 1], port), async move {
            rx.await.ok();
        });

        (tx, tokio::spawn(server))
    }

    #[tokio::test]
    async fn requests_are_parsed_correctly()
    {
        let request = GetRoomPropertiesRequest {
            room_id: 1,
            dates: vec![
                GregorianScheduleDate::new(2023, 4, 29),
                GregorianScheduleDate::new(2023, 4, 30),
                GregorianScheduleDate::new(2023, 5, 1),
            ],
            entries: false,
            users: false,
            display_name: false,
            url: false,
            broadcast: false,
        };

        let (tx, server) = server_expecting_request(8001, Base64EncodedRequest(request));

        let client = Client::new();
        let res = client
            .get(Uri::from_static(
                "http://127.0.0.1:8001/api/room/get-room-properties?b64=eyJyb29tSWQiOjEsImRhdGVzIjpbeyJ5ZWFyIjoyMDIzLCJtb250aCI6NCwiZGF5IjoyOX0seyJ5ZWFyIjoyMDIzLCJtb250aCI6NCwiZGF5IjozMH0seyJ5ZWFyIjoyMDIzLCJtb250aCI6NSwiZGF5IjoxfV0sImVudHJpZXMiOmZhbHNlLCJ1c2VycyI6ZmFsc2UsImRpc3BsYXlOYW1lIjpmYWxzZSwidXJsIjpmYWxzZSwiYnJvYWRjYXN0IjpmYWxzZX0",
            ))
            .await
            .unwrap();
        println!("status: {}", res.status());
        println!("body: {:?}", res.body());

        assert!(res.status().is_success());

        tx.send(()).unwrap();
        server.await.ok();
    }
}
