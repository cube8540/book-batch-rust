use crate::item::Site;
use mongodb::bson::doc;
use mongodb::sync::Client;
use serde::de::Visitor;
use serde::{Deserializer, Serializer};
use serde_with::serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, DeserializeAs, SerializeAs};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ConnectError(String),

    SqlExecuteError(String),

    ConvertError(String),
}

struct SiteFromStr {}

impl SerializeAs<Site> for SiteFromStr {
    fn serialize_as<S>(source: &Site, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_str(&source.to_code_str())
    }
}

impl <'de> DeserializeAs<'de, Site> for SiteFromStr {
    fn deserialize_as<D>(deserializer: D) -> Result<Site, D::Error>
    where
        D: Deserializer<'de>
    {
        struct Helper(PhantomData<Site>);
        impl Visitor<'_> for Helper {
            type Value = Site;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a Site enum variant")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Site::from_str(v).map_err(|e| serde::de::Error::custom(e.to_string()))
            }
        }
        deserializer.deserialize_str(Helper(PhantomData))
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookOriginData {
    book_id: i64,

    #[serde_as(as = "SiteFromStr")]
    site: Site,

    #[serde(flatten)]
    original: HashMap<String, String>,
}

pub struct BookOriginDataStore {
    client: Client,
}

impl BookOriginDataStore {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl BookOriginDataStore {

    pub fn find_by_book_id(&self, book_id: &[i64]) -> Result<Vec<BookOriginData>, Error> {
        let filter = doc! { "book_id": { "$in": book_id } };

        let collection = self.client
            .database("workspace")
            .collection::<BookOriginData>("book_origin_data");

        let result = collection.find(filter).run()
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        let docs = result
            .collect::<Result<Vec<BookOriginData>, mongodb::error::Error>>()
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(docs)
    }
}
