use serde::{
    Deserialize,
    Serialize,
};

pub type UserID = String;

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct GregorianScheduleDate
{
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct EntryKey
{
    user_id: UserID,
    schedule_date: GregorianScheduleDate,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct EntryData
{
    amount: u32,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct Entry
{
    key: EntryKey,
    value: EntryData,
}
