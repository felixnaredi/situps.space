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

#[derive(Debug, PartialEq, Clone)]
struct Base64EncodedRequest<T>(T);

impl<T: Serialize> Serialize for Base64EncodedRequest<T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(
            "b64",
            &base64::engine::general_purpose::STANDARD_NO_PAD
                .encode(serde_json::to_string(&self.0).unwrap()),
        )?;
        map.end()
    }
}

#[derive(Deserialize)]
struct Field
{
    b64: String,
}

struct Base64EncodedRequestVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for Base64EncodedRequestVisitor<T>
{
    type Value = Base64EncodedRequest<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(formatter, "struct Base64EncodedRequest<T>")?;
        Ok(())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Base64EncodedRequest<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["b64"];
        deserializer.deserialize_struct(
            "Base64EncodedRequest",
            FIELDS,
            Base64EncodedRequestVisitor::<T>(PhantomData),
        )
    }
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
    use hyper::{
        Client,
        Uri,
    };
    use serde::de::DeserializeOwned;
    use tokio::{
        sync::oneshot,
        task::JoinError,
    };
    use warp::reply::Response;

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
                    println!("{:#?}", request);
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
        println!("channel created");

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

        let (tx, server) = server_expecting_request(
            8001,
            Base64EncodedRequest(request.clone()),
        );

        println!(
            "{}",
            serde_urlencoded::to_string(&Base64EncodedRequest(request)).unwrap()
        );

        let client = Client::new();
        let res = client
            .get(Uri::from_static(
                "http://127.0.0.1:8001/api/room/get-room-properties?b64=eyJyb29tSWQiOjEsImRhdGVzIjpbeyJ5ZWFyIjoyMDIzLCJtb250aCI6NCwiZGF5IjoyOX0seyJ5ZWFyIjoyMDIzLCJtb250aCI6NCwiZGF5IjozMH0seyJ5ZWFyIjoyMDIzLCJtb250aCI6NSwiZGF5IjoxfV0sImVudHJpZXMiOmZhbHNlLCJ1c2VycyI6ZmFsc2UsImRpc3BsYXlOYW1lIjpmYWxzZSwidXJsIjpmYWxzZSwiYnJvYWRjYXN0IjpmYWxzZX0",
            ))
            .await
            .unwrap();
        println!("status: {}", res.status());
        println!("body: {:?}", res.body());

        tx.send(()).unwrap();
        server.await.ok();
    }
}
