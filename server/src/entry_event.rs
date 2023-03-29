use serde::{
    Deserialize,
    Serialize,
};
use warp::ws::Message;

use crate::schemes::EntryData;

pub mod request
{
    use super::*;
    use crate::schemes::EntryKey;

    #[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetEntryData
    {
        pub entry_key: EntryKey,
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Request
{
    GetEntryData(request::GetEntryData),
    Update(),
}

pub mod response
{
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetEntryData
    {
        entry_data: Option<EntryData>,
    }

    impl GetEntryData
    {
        pub fn new(entry_data: Option<EntryData>) -> GetEntryData
        {
            GetEntryData { entry_data }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Response
{
    ConnectionEstablished,
    GetEntryData(response::GetEntryData),
    Update(),
}

impl Response
{
    pub fn connection_established() -> Message {
        Message::text(serde_json::to_string(&Response::ConnectionEstablished).unwrap()
        )
    }

    pub fn get_entry_data(entry_data: Option<EntryData>) -> Message
    {
        Message::text(
            serde_json::to_string(&Response::GetEntryData(response::GetEntryData::new(entry_data))).unwrap(),
        )
    }
}
