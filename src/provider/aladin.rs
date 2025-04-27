use crate::provider::error::ClientError;
use crate::provider::Request;
use crate::{book, provider};
use reqwest::blocking;
use serde::Deserialize;
use std::collections::HashMap;

/// 알라딘 API 엔드포인트 URL
const ALADIN_API_ENDPOINT: &'static str = "https://www.aladin.co.kr/ttb/api/ItemSearch.aspx";
/// API 요청의 기본 타임아웃 시간(초)
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

pub const SITE: &'static str = "ALADIN";

/// 알라딘 API 응답을 표현하는 구조체
#[derive(Debug, Deserialize)]
pub struct AladinResponse {
    /// API 버전
    pub version: String,
    /// 검색 결과 제목
    #[serde(rename = "title")]
    pub title: String,
    /// 검색 결과 링크
    #[serde(rename = "link")]
    pub link: String,
    /// 검색 결과 발행일
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    /// 총 결과 수
    #[serde(rename = "totalResults")]
    pub total_results: i32,
    /// 시작 인덱스
    #[serde(rename = "startIndex")]
    pub start_index: i32,
    /// 페이지당 아이템 수
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: i32,
    /// 검색 쿼리
    #[serde(rename = "query")]
    pub query: String,
    /// 검색 카테고리 ID
    #[serde(rename = "searchCategoryId")]
    pub search_category_id: i32,
    /// 검색 카테고리 이름
    #[serde(rename = "searchCategoryName")]
    pub search_category_name: String,
    /// 도서 아이템 목록
    #[serde(rename = "item")]
    pub items: Vec<BookItem>,

}

/// 개별 도서 정보를 표현하는 구조체
#[derive(Debug, Deserialize)]
pub struct BookItem {
    /// 도서 제목
    #[serde(rename = "title")]
    pub title: String,
    /// 도서 상세 페이지 링크
    #[serde(rename = "link")]
    pub link: String,
    /// 저자 정보
    #[serde(rename = "author")]
    pub author: String,
    /// 출판일
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    /// 도서 설명
    #[serde(rename = "description")]
    pub description: String,
    /// ISBN 코드(10자리)
    #[serde(rename = "isbn")]
    pub isbn: String,
    /// ISBN13 코드(13자리)
    #[serde(rename = "isbn13")]
    pub isbn13: String,
    /// 알라딘 도서 ID
    #[serde(rename = "itemId")]
    pub item_id: i64,
    /// 판매 가격
    #[serde(rename = "priceSales")]
    pub price_sales: i32,
    /// 정가
    #[serde(rename = "priceStandard")]
    pub price_standard: i32,
    /// 출판사
    #[serde(rename = "publisher")]
    pub publisher: String,
}

impl BookItem {
    fn to_map(&self) -> HashMap<String, String> {
        std::collections::HashMap::from([
            ("title".to_string(), self.title.clone()),
            ("link".to_string(), self.link.clone()),
            ("author".to_string(), self.author.clone()),
            ("pub_date".to_string(), self.pub_date.clone()),
            ("description".to_string(), self.description.clone()),
            ("isbn".to_string(), self.isbn.clone()),
            ("isbn13".to_string(), self.isbn13.clone()),
            ("item_id".to_string(), self.item_id.to_string()),
            ("price_sales".to_string(), self.price_sales.to_string()),
            ("price_standard".to_string(), self.price_standard.to_string()),
            ("publisher".to_string(), self.publisher.clone()),
        ])
    }
}

/// 알라딘 API 클라이언트
pub struct Client {
    /// 알라딘 API TTB 키
    ttb_key: String,
}


impl Client {
    pub fn new(ttb_key: &str) -> Self {
        Client {
            ttb_key: ttb_key.to_string(),
        }
    }

    fn build_search_url(&self, request: &Request) -> Result<reqwest::Url, ClientError> {
        let mut url = reqwest::Url::parse(ALADIN_API_ENDPOINT)
            .map_err(|_| ClientError::InvalidBaseUrl)?;

        url.query_pairs_mut()
            .append_pair("ttbkey", &self.ttb_key)
            .append_pair("Query", &request.query.clone())
            .append_pair("QueryType", "Publisher")  // Publisher로 고정
            .append_pair("start", &request.page.to_string())
            .append_pair("MaxResults", &request.size.to_string())
            .append_pair("SearchTarget", "Book")  // Book으로 고정
            .append_pair("output", "js") // JS로 고정
            .append_pair("Version", "20131101")
            .append_pair("Sort", "PublishTime");

        Ok(url)
    }
}

impl provider::Client for Client {
    fn get_books(&self, request: &Request) -> Result<provider::Response, ClientError> {
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

        let parsed_response = serde_json::from_str::<AladinResponse>(&text)
            .map_err(|err| ClientError::ResponseParseFailed(err.to_string()))?;

        let books = parsed_response.items.iter()
            .map(|item| convert_item_to_book(item));

        Ok(provider::Response{
            total_count: parsed_response.total_results,
            page_no: parsed_response.start_index,
            site: SITE.to_string(),
            books: books.collect(),
        })
    }
}

fn convert_item_to_book(item: &BookItem) -> book::Book {
    book::Book {
        id: 0,
        isbn: item.isbn13.clone(),
        publisher_id: 0,
        title: item.title.clone(),
        scheduled_pub_date: None,
        actual_pub_date: chrono::NaiveDate::parse_from_str(item.pub_date.as_str(), "%Y-%m-%d").ok(),
        origin_data: HashMap::from([(SITE.to_string(), item.to_map())]),
    }
}