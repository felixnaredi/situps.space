use serde::{
    Deserialize,
    Serialize,
};

pub type UserID = String;

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GregorianScheduleDate
{
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryKey
{
    #[serde(rename = "userId")]
    id: UserID,
    schedule_date: GregorianScheduleDate,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryData
{
    pub amount: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry
{
    #[serde(rename = "_id")]
    id: EntryKey,
    value: EntryData,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User
{
    #[serde(rename = "_id")]
    id: UserID,
    display_name: String,
    theme: String,
}
