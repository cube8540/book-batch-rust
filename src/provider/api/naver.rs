use crate::item::{Book, Site};
use crate::provider::api::{ClientError, Request, Response};
use crate::provider;
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

impl RssResponse {
    pub fn into_response(self) -> Response {
        if let Some(channel) = self.channel {
            let item = channel.item.unwrap_or_else(|| vec![]);
            let books = item.into_iter()
                .map(|item| {
                    let actual_pub_date = if !item.pubdate.is_empty() {
                        chrono::NaiveDate::parse_from_str(&item.pubdate, "%Y%m%d").ok()
                    } else {
                        None
                    };
                    let mut builder = Book::builder()
                        .isbn(item.isbn.clone())
                        .title(item.title.clone())
                        .add_original(Site::Naver, item.to_map());
                    if let Some(pub_date) = actual_pub_date {
                        builder = builder.actual_pub_date(pub_date);
                    }
                    builder
                })
                .collect();

            Response {
                total_count: channel.total,
                page_no: channel.start,
                site: Site::Naver,
                books,
            }
        } else {
            Response::empty(Site::Naver)
        }
    }
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
    pub fn to_map(&self) -> HashMap<String, String> {
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
}

pub struct Client {
    client_id: String,
    client_secret: String,
}

pub fn new_client() -> Result<Client, VarError> {
    let client_id = std::env::var("NAVER_KEY")?;
    let client_secret = std::env::var("NAVER_SECRET")?;

    Ok(Client {
        client_id,
        client_secret,
    })
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

        Ok(parsed_response.into_response())
    }
}