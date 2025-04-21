use crate::external::error::{ClientError, RequestError};
use reqwest::blocking;
use serde::Deserialize;

const ALADIN_API_ENDPOINT: &str = "https://www.aladin.co.kr/ttb/api/ItemSearch.aspx";
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_PAGE: i32 = 1;
const DEFAULT_SIZE: i32 = 10;

#[derive(Debug, Deserialize)]
pub struct AladinResponse {
    pub version: String,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "link")]
    pub link: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    #[serde(rename = "totalResults")]
    pub total_results: i32,
    #[serde(rename = "startIndex")]
    pub start_index: i32,
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: i32,
    #[serde(rename = "query")]
    pub query: String,
    #[serde(rename = "searchCategoryId")]
    pub search_category_id: i32,
    #[serde(rename = "searchCategoryName")]
    pub search_category_name: String,
    #[serde(rename = "item")]
    pub items: Vec<BookItem>,
}

#[derive(Debug, Deserialize)]
pub struct BookItem {
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "link")]
    pub link: String,
    #[serde(rename = "author")]
    pub author: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "isbn")]
    pub isbn: String,
    #[serde(rename = "isbn13")]
    pub isbn13: String,
    #[serde(rename = "itemId")]
    pub item_id: i64,
    #[serde(rename = "priceSales")]
    pub price_sales: i32,
    #[serde(rename = "priceStandard")]
    pub price_standard: i32,
    #[serde(rename = "stockStatus")]
    pub stock_status: String,
    #[serde(rename = "mileage")]
    pub mileage: i32,
    #[serde(rename = "categoryId")]
    pub category_id: i32,
    #[serde(rename = "categoryName")]
    pub category_name: String,
    #[serde(rename = "publisher")]
    pub publisher: String,
    #[serde(rename = "customerReviewRank")]
    pub customer_review_rank: i32,
}

pub struct Request {
    query: String,
    start: i32,
    max_results: i32,
}

pub struct RequestBuilder {
    query: Option<String>,
    start: i32,
    max_results: i32,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder {
            query: None,
            start: DEFAULT_PAGE,
            max_results: DEFAULT_SIZE,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn start(&self) -> i32 {
        self.start
    }

    pub fn max_results(&self) -> i32 {
        self.max_results
    }
}

impl RequestBuilder {
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn start(mut self, start: i32) -> Self {
        self.start = start;
        self
    }

    pub fn max_results(mut self, max_results: i32) -> Self {
        self.max_results = max_results;
        self
    }

    pub fn build(self) -> Result<Request, RequestError> {
        let query = self.query.ok_or_else(||
            RequestError::MissingRequiredParameter("query".to_string()))?;
        if query.trim().is_empty() {
            return Err(RequestError::InvalidParameter("검색어는 비어있을 수 없습니다".to_string()));
        }
        Ok(Request {
            query,
            start: self.start,
            max_results: self.max_results,
        })
    }
}

pub struct Client {
    ttb_key: String,
}

impl Client {
    pub fn new(ttb_key: impl Into<String>) -> Self {
        Client {
            ttb_key: ttb_key.into(),
        }
    }

    pub fn get_books(&self, request: Request) -> Result<AladinResponse, ClientError> {
        let client = blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .build()
            .map_err(|e| ClientError::RequestFailed(format!("클라이언트 생성 실패: {}", e)))?;

        let url = self.build_search_url(&request)?;

        let response = client.get(url)
            .send()
            .map_err(|err| ClientError::RequestFailed(err.to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::RequestFailed(format!("HTTP 오류: {}", response.status())));
        }

        let text = response.text()
            .map_err(|err| ClientError::ResponseTextExtractionFailed(err.to_string()))?;

        serde_json::from_str::<AladinResponse>(&text)
            .map_err(|err| ClientError::ResponseParseFailed(err.to_string()))
    }

    fn build_search_url(&self, request: &Request) -> Result<reqwest::Url, ClientError> {
        let mut url = reqwest::Url::parse(ALADIN_API_ENDPOINT)
            .map_err(|_| ClientError::InvalidBaseUrl)?;

        url.query_pairs_mut()
            .append_pair("ttbkey", &self.ttb_key)
            .append_pair("Query", &request.query())
            .append_pair("QueryType", "Publisher")  // Publisher로 고정
            .append_pair("start", &request.start().to_string())
            .append_pair("MaxResults", &request.max_results().to_string())
            .append_pair("SearchTarget", "Book")  // Book으로 고정
            .append_pair("output", "js") // JS로 고정
            .append_pair("Version", "20131101")
            .append_pair("Sort", "PublishTime");

        Ok(url)
    }
}