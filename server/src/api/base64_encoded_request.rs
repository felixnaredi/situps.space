use std::marker::PhantomData;

use base64::Engine as _;
use serde::{
    de::{
        DeserializeOwned,
        MapAccess,
        Visitor,
    },
    ser::SerializeMap,
    Deserialize,
    Serialize,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Base64EncodedRequest<T>(pub T);

impl<T> Serialize for Base64EncodedRequest<T>
where
    T: Serialize,
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

struct Base64EncodedRequestVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for Base64EncodedRequestVisitor<T>
where
    T: DeserializeOwned,
{
    type Value = Base64EncodedRequest<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(formatter, "struct Base64EncodedRequest<T>")?;
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // TODO:
        //   Avoid using `unwrap`. There should be ways to return fitting `Err`.
        match map.next_entry::<&str, &str>()? {
            Some(("b64", data)) => {
                //
                // 3. Add the `Base64EncodeRequest` wrapper.
                //
                Ok(Base64EncodedRequest(
                    //
                    // 2. Deserialize the JSON object.
                    //
                    serde_json::from_slice(
                        //
                        // 1. Decode base64 string.
                        //
                        &base64::engine::general_purpose::STANDARD_NO_PAD
                            .decode(data)
                            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?,
                    )
                    .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?,
                ))
            }
            _ => Err(serde::de::Error::missing_field("b64")),
        }
    }
}

impl<'de, T> Deserialize<'de> for Base64EncodedRequest<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["b64"];
        deserializer.deserialize_struct(
            "Base64EncodedRequest",
            FIELDS,
            Base64EncodedRequestVisitor(PhantomData),
        )
    }
}
