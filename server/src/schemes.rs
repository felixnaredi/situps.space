use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};

pub type UserID = String;

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize, Hash, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GregorianScheduleDate
{
    year: i32,
    month: u32,
    day: u32,
}

impl GregorianScheduleDate
{
    pub fn new(year: i32, month: u32, day: u32) -> GregorianScheduleDate
    {
        GregorianScheduleDate { year, month, day }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryKey
{
    pub user_id: UserID,
    pub schedule_date: GregorianScheduleDate,
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
    pub id: EntryKey,
    pub value: EntryData,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User
{
    #[serde(rename = "_id")]
    user_id: UserID,
    display_name: String,
    theme: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntryCommit
{
    pub date: DateTime<Utc>,
    pub data: Entry,
}
