use crate::provider::api::{ClientError, Request, Response};
use crate::{book, provider};
use serde::Deserialize;
use std::collections::HashMap;
use serde_with::serde_as;

const BOOK_SEARCH_ENDPOINT: &'static str = "https://openapi.naver.com/v1/search/book_adv.xml";

pub const SITE: &'static str = "NAVER";

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RssResponse {
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "channel")]
    pub channel: Channel,

}

impl RssResponse {
    pub fn into_response(self) -> Response {
        let books = self.channel.item.into_iter()
            .map(|item| {
                let actual_pub_date = if !item.pubdate.is_empty() {
                    chrono::NaiveDate::parse_from_str(&item.pubdate, "%Y%m%d").ok()
                } else {
                    None
                };
                book::Book {
                    id: 0,
                    isbn: item.isbn.clone(),
                    publisher_id: 0,
                    title: item.title.clone(),
                    scheduled_pub_date: None,
                    actual_pub_date,
                    origin_data: HashMap::from([(SITE.to_string(), item.to_map())]),
                }
            })
            .collect();

        Response {
            total_count: self.channel.total,
            page_no: self.channel.start,
            site: SITE.to_string(),
            books,
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
    pub item: Vec<Item>,

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
    pub discount: i32,
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
        let fields = [
            ("title", self.title.to_string()),
            ("link", self.link.to_string()),
            ("image", self.image.to_string()),
            ("author", self.author.to_string()),
            ("discount", self.discount.to_string()),
            ("publisher", self.publisher.to_string()),
            ("pubdate", self.pubdate.to_string()),
            ("isbn", self.isbn.to_string()),
            ("description", self.description.to_string()),
        ];

        fields
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    }
}

pub struct Client {
    client_id: String,
    client_secret: String,
}

pub fn new(client_id: String, client_secret: String) -> Client {
    Client {
        client_id,
        client_secret,
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
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;
        let response_text = response.text()
            .map_err(|e| ClientError::ResponseTextExtractionFailed(e.to_string()))?;
        let parsed_response: RssResponse = serde_xml_rs::from_str(&response_text)
            .map_err(|e| ClientError::ResponseParseFailed(e.to_string()))?;

        Ok(parsed_response.into_response())
    }
}
