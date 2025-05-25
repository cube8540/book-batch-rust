use crate::item::{Originals, Raw, Site};
use mongodb::bson::doc;
use mongodb::sync::Client;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    original: HashMap<String, serde_json::Value>,
}

impl BookOriginData {

    pub fn to_domain(&self) -> (Site, Raw) {
        let original = self.original.iter()
            .filter_map(|(k, v)| {
                v.as_str().map(|v| (k.clone(), v.into()))
            })
            .collect::<HashMap<_, _>>();

        (self.site, original)
    }
}

impl BookOriginData {
    pub fn new(book_id: i64, site: Site) -> Self {
        Self {
            book_id,
            site,
            original: HashMap::new(),
        }
    }

    pub fn book_id(&self) -> i64 {
        self.book_id
    }

    pub fn site(&self) -> Site {
        self.site
    }

    pub fn original(&self) -> &HashMap<String, serde_json::Value> {
        &self.original
    }

    pub fn add_origin(&mut self, key: &str, value: serde_json::Value) {
        self.original.insert(key.to_owned(), value);
    }
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

    pub fn new_original_data(&self, book_id: i64, origins: &Originals) -> Result<usize, Error> {
        let docs = origins.into_iter()
            .map(|(site, raw)| {
                let mut origin = BookOriginData::new(book_id, site.clone());
                for (k, v) in raw {
                    origin.add_origin(k, v.to_serde_json());
                }
                origin
            });

        let collection = self.client
            .database("workspace")
            .collection::<BookOriginData>("book_origin_data");

        let results = collection.insert_many(docs).run()
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(results.inserted_ids.len())
    }

    pub fn delete_site(&self, book_id: i64, site: &Site) -> Result<usize, Error> {
        let doc = doc! { "book_id": book_id, "site": site.to_code_str() };

        let collection = self.client
            .database("workspace")
            .collection::<BookOriginData>("book_origin_data");

        let results = collection.delete_many(doc).run()
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(results.deleted_count as usize)
    }
}
