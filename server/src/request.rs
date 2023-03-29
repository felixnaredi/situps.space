pub mod entry_event {
    use serde::{Deserialize, Serialize};

    use crate::schemes::EntryKey;

    #[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Get {
        entry_key: EntryKey,
    }
}