use crate::external::error::{ClientError, RequestError};
use reqwest::blocking;
use serde::Deserialize;

/// 알라딘 API 엔드포인트 URL
const ALADIN_API_ENDPOINT: &str = "https://www.aladin.co.kr/ttb/api/ItemSearch.aspx";
/// API 요청의 기본 타임아웃 시간(초)
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;
/// 기본 페이지 번호
const DEFAULT_PAGE: i32 = 1;
/// 기본 페이지당 결과 개수
const DEFAULT_SIZE: i32 = 10;

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

/// API 요청 정보를 담는 구조체
pub struct Request {
    /// 검색 쿼리
    query: String,
    /// 시작 인덱스
    start: i32,
    /// 최대 결과 수
    max_results: i32,
}

/// API 요청 빌더 구조체
pub struct RequestBuilder {
    /// 검색 쿼리 (선택적)
    query: Option<String>,
    /// 시작 인덱스
    start: i32,
    /// 최대 결과 수
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

/// 알라딘 API 클라이언트
pub struct Client {
    /// 알라딘 API TTB 키
    ttb_key: String,
}

impl Client {
    pub fn new(ttb_key: impl Into<String>) -> Self {
        Client {
            ttb_key: ttb_key.into(),
        }
    }

    /// 도서 정보 검색 수행
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