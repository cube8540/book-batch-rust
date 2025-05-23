use crate::item::{Book, BookBuilder, Site};
use crate::provider;
use crate::provider::api::{ClientError, Request, Response};
use serde::Deserialize;
use serde_with::serde_as;
use std::collections::HashMap;
use std::env::VarError;

const BOOK_SEARCH_ENDPOINT: &'static str = "https://openapi.naver.com/v1/search/book_adv.xml";

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RssResponse {
    #[serde(rename = "channel")]
    pub channel: Option<Channel>,

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "link")]
    pub link: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "lastBuildDate")]
    pub last_build_date: String,
    #[serde(rename = "total")]
    pub total: i32,
    #[serde(rename = "start")]
    pub start: i32,
    #[serde(rename = "display")]
    pub display: i32,
    #[serde(rename = "item")]
    pub item: Option<Vec<Item>>,

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Item {
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "link")]
    pub link: String,
    #[serde(rename = "image")]
    pub image: String,
    #[serde(rename = "author")]
    pub author: String,
    #[serde(rename = "discount")]
    pub discount: Option<i32>,
    #[serde(rename = "publisher")]
    pub publisher: String,
    #[serde(rename = "pubdate")]
    pub pubdate: String,
    #[serde(rename = "isbn")]
    pub isbn: String,
    #[serde(rename = "description")]
    pub description: String,

}

impl Item {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        map.insert("title".to_string(), self.title.clone());
        map.insert("link".to_string(), self.link.clone());
        map.insert("image".to_string(), self.image.clone());
        map.insert("author".to_string(), self.author.clone());
        map.insert("publisher".to_string(), self.publisher.clone());
        map.insert("pubdate".to_string(), self.pubdate.clone());
        map.insert("isbn".to_string(), self.isbn.clone());
        map.insert("description".to_string(), self.description.clone());

        if let Some(discount) = self.discount {
            map.insert("discount".to_string(), discount.to_string());
        }
        
        map
    }

    fn to_book_builder(&self) -> BookBuilder {
        let mut builder = Book::builder()
            .isbn(self.isbn.clone())
            .title(self.title.clone())
            .add_original(Site::Naver, self.to_map());

        let actual_pub_date = if !self.pubdate.is_empty() {
            chrono::NaiveDate::parse_from_str(&self.pubdate, "%Y%m%d").ok()
        } else {
            None
        };
        if let Some(pub_date) = actual_pub_date {
            builder = builder.actual_pub_date(pub_date);
        }
        builder
    }
}

#[derive(Clone)]
pub struct Client {
    client_id: String,
    client_secret: String,
}

impl Client {
    pub fn new_with_env() -> Result<Client, VarError> {
        let client_id = std::env::var("NAVER_KEY")?;
        let client_secret = std::env::var("NAVER_SECRET")?;

        Ok(Self { client_id, client_secret })
    }
}

impl provider::api::Client for Client {

    fn get_books(&self, request: &Request) -> Result<Response, ClientError> {
        let mut url = reqwest::Url::parse(BOOK_SEARCH_ENDPOINT).unwrap();
        url.query_pairs_mut()
            .append_pair("d_isbn", request.query.as_str());

        let client = reqwest::blocking::Client::new()
            .get(url)
            .header("X-Naver-Client-Id", self.client_id.as_str())
            .header("X-Naver-Client-Secret", self.client_secret.as_str());

        let response = client.send()
            .map_err(|e| ClientError::RequestFailed(format!("ISBN: {}, ERROR: {:?}", request.query, e)))?;
        let response_text = response.text()
            .map_err(|e| ClientError::ResponseTextExtractionFailed(format!("ISBN: {}, ERROR: {:?}", request.query, e)))?;
        let parsed_response: RssResponse = serde_xml_rs::from_str(&response_text)
            .map_err(|e| ClientError::ResponseParseFailed(format!("ISBN: {}, ERROR: {:?}", request.query, e)))?;

        let response = parsed_response.channel
            .map(|channel| {
                let books = channel.item.unwrap_or_else(|| vec![]).into_iter()
                    .map(|item| item.to_book_builder())
                    .collect::<Vec<BookBuilder>>();

                Response {
                    total_count: channel.total,
                    page_no: channel.start,
                    site: Site::Naver,
                    books,
                }
            })
            .unwrap_or_else(|| Response::empty(Site::Naver));

        Ok(response)
    }
}