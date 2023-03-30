use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson,
    bson::doc,
    options::{
        FindOneOptions,
        FindOptions,
        UpdateOptions,
    },
    Database,
};

use crate::schemes::{
    Entry,
    EntryData,
    EntryKey,
    UpdateEntryCommit,
    User,
};

pub async fn get_users(db: &Database) -> Result<Vec<User>, Box<dyn std::error::Error>>
{
    let cursor = db
        .collection::<User>("users")
        .find(
            doc! {},
            FindOptions::builder()
                .projection(doc! {
                    "_id": "$_id", "displayName": 1, "theme": 1
                })
                .build(),
        )
        .await?;
    Ok(cursor.try_collect::<Vec<_>>().await?)
}

pub async fn get_entry_data(
    db: &Database,
    entry_key: &EntryKey,
) -> Result<Option<EntryData>, Box<dyn std::error::Error>>
{
    log::debug!("bson: {:?}", bson::to_bson(&entry_key).unwrap());
    Ok(db
        .collection::<EntryData>("entries")
        .find_one(
            doc! { "_id": bson::to_bson(&entry_key).unwrap(), "amount": { "$gt": 0 }},
            FindOneOptions::builder()
                .projection(doc! { "_id": 0, "amount": 1})
                .build(),
        )
        .await?)
}

pub async fn update_entry(db: &Database, entry: &Entry) -> Result<(), Box<dyn std::error::Error>>
{
    log::debug!("update entry - {:?}", entry);

    //
    // Log commit.
    //
    db.collection::<UpdateEntryCommit>("updateEntryCommits")
        .insert_one(
            UpdateEntryCommit {
                date: Utc::now(),
                data: entry.clone(),
            },
            None,
        )
        .await?;

    //
    // Update database.
    //
    db.collection::<Entry>("entries")
        .update_one(
            doc! {
                "_id": bson::to_bson(&entry.id).unwrap(),
            },
            doc! {
                "$set": { "amount": &entry.value.amount },
            },
            UpdateOptions::builder().upsert(true).build(),
        )
        .await?;

    Ok(())
}
