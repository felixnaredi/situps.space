use serde::{
    Deserialize,
    Serialize,
};
use warp::ws::Message;

use crate::schemes::{
    Entry,
    EntryData,
    EntryKey,
};

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

    #[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateEntry
    {
        pub entry: Entry,
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Request
{
    GetEntryData(request::GetEntryData),
    UpdateEntry(request::UpdateEntry),
}

pub mod response
{
    use crate::schemes::EntryKey;

    use super::*;

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetEntryData {
        entry_key: EntryKey, 
        entry_data: Option<EntryData>
    }

    impl GetEntryData
    {
        pub fn new(entry_key: EntryKey, entry_data: Option<EntryData>) -> GetEntryData
        {
            GetEntryData { entry_key, entry_data }
        }
    }

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateEntry(Entry);

    impl UpdateEntry
    {
        pub fn new(entry: Entry) -> UpdateEntry
        {
            UpdateEntry(entry)
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Response
{
    ConnectionEstablished,
    GetEntryData(response::GetEntryData),
    UpdateEntry(response::UpdateEntry),
}

impl Response
{
    pub fn connection_established() -> Message
    {
        Message::text(serde_json::to_string(&Response::ConnectionEstablished).unwrap())
    }

    pub fn get_entry_data(entry_key: EntryKey, entry_data: Option<EntryData>) -> Message
    {
        Message::text(
            serde_json::to_string(&Response::GetEntryData(response::GetEntryData::new(
                entry_key,
                entry_data,
            )))
            .unwrap(),
        )
    }

    pub fn update_entry(entry: Entry) -> Message
    {
        Message::text(
            serde_json::to_string(&Response::UpdateEntry(response::UpdateEntry::new(entry)))
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Broadcast
{
    UpdateEntry(Entry),
}
