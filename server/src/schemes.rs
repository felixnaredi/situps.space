use serde::{
    Deserialize,
    Serialize,
};

pub type UserID = String;

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GregorianScheduleDate
{
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryKey
{
    user_id: UserID,
    schedule_date: GregorianScheduleDate,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryData
{
    amount: u32,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry
{
    #[serde(rename = "_id")]
    id: EntryKey,
    value: EntryData,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User
{
    #[serde(rename = "_id")]
    id: UserID,
    display_name: String,
    theme: String,
}
