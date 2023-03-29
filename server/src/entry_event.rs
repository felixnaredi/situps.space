use serde::{
    Deserialize,
    Serialize,
};

use crate::schemes::EntryData;

pub mod request
{
    use super::*;
    use crate::schemes::EntryKey;

    #[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Get
    {
        pub entry_key: EntryKey,
    }
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Request
{
    Get(request::Get),
    Update(),
}

pub mod response
{
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Get
    {
        entry_data: Option<EntryData>,
    }

    impl Get {
        pub fn new(entry_data: Option<EntryData>) -> Get {
            Get { entry_data }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Response
{
    Get(response::Get),
    Update(),
}

impl Response {
    pub fn get(entry_data: Option<EntryData>) -> Response {
        Response::Get(response::Get::new(entry_data))
    }
}