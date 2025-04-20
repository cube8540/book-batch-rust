use crate::external::error::ClientError;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use chrono::NaiveDate;

/// 국립중앙도서관 ISBN 도서정보 검색 API 엔드포인트 URL
const ISBN_SEARCH_ENDPOINT: &'static str = "https://www.nl.go.kr/seoji/SearchApi.do";
/// API 요청 시 기본 타임아웃 시간(초)
const DEFAULT_TIMEOUT_SECONDS: i64 = 10;
/// 검색 결과 기본 페이지 번호
const DEFAULT_PAGE: i32 = 1;
/// 페이지당 기본 결과 수
const DEFAULT_SIZE: i32 = 50;

/// 국립중앙도서관 API에서 반환하는 도서 정보 구조체
#[derive(Deserialize)]
pub struct Doc {
    /// 도서 제목
    #[serde(rename = "TITLE")]
    pub title: String,
    /// 단권 ISBN
    #[serde(rename = "EA_ISBN")]
    pub ea_isbn: String,
    /// 세트 ISBN
    #[serde(rename = "SET_ISBN")]
    pub set_isbn: String,
    /// 부가 코드(단권)
    #[serde(rename = "EA_ADD_CODE")]
    pub ea_add_code: String,
    /// 부가 코드(세트)
    #[serde(rename = "SET_ADD_CODE")]
    pub set_add_code: String,
    /// 시리즈 번호
    #[serde(rename = "SERIES_NO")]
    pub series_no: String,
    /// 세트 표현
    #[serde(rename = "SET_EXPRESSION")]
    pub set_expression: String,
    /// 주제 분류
    #[serde(rename = "SUBJECT")]
    pub subject: String,
    /// 출판사
    #[serde(rename = "PUBLISHER")]
    pub publisher: String,
    /// 저자
    #[serde(rename = "AUTHOR")]
    pub author: String,
    /// 실제 출판일
    #[serde(rename = "REAL_PUBLISH_DATE")]
    pub real_publish_date: String,
    /// 예정 출판일
    #[serde(rename = "PUBLISH_PREDATE")]
    pub publish_predate: String,
    /// 데이터 갱신일
    #[serde(rename = "UPDATE_DATE")]
    pub update_date: String,
}

/// API 응답 구조체로 검색 결과 메타데이터와 도서 정보 목록 포함
#[serde_as]
#[derive(Deserialize)]
pub struct Response {
    /// 검색된 총 도서 수
    #[serde(rename = "TOTAL_COUNT")]
    #[serde_as(as = "DisplayFromStr")]
    pub total_count: i32,

    /// 현재 페이지 번호
    #[serde(rename = "PAGE_NO")]
    #[serde_as(as = "DisplayFromStr")]
    pub page_no: i32,

    /// 검색된 도서 목록
    pub docs: Vec<Doc>,
}

/// API 요청 매개변수를 담는 구조체
pub struct Request {
    /// 요청 페이지 번호
    page: i32,
    /// 페이지당 결과 수
    size: i32,
    /// 출판사 필터
    publisher: String,
    /// 출판일 시작 범위
    start_pub_date: NaiveDate,
    /// 출판일 종료 범위
    end_pub_date: NaiveDate,
}

/// 빌더 패턴을 구현한 요청 빌더 구조체
pub struct RequestBuilder {
    /// 요청 페이지 번호 (기본값 사용)
    page: i32,
    /// 페이지당 결과 수 (기본값 사용)
    size: i32,
    /// 출판사 필터 (선택사항)
    publisher: Option<String>,
    /// 출판일 시작 범위 (선택사항)
    start_pub_date: Option<NaiveDate>,
    /// 출판일 종료 범위 (선택사항)
    end_pub_date: Option<NaiveDate>,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder {
            page: DEFAULT_PAGE,
            size: DEFAULT_SIZE,
            publisher: None,
            start_pub_date: None,
            end_pub_date: None,
        }
    }

    pub fn page(&self) -> i32 {
        self.page
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn publisher(&self) -> &str {
        &self.publisher
    }

    pub fn start_pub_date(&self) -> NaiveDate {
        self.start_pub_date
    }

    pub fn end_pub_date(&self) -> NaiveDate {
        self.end_pub_date
    }
}

impl RequestBuilder {
    pub fn page(mut self, page: i32) -> Self {
        self.page = page;
        self
    }

    pub fn size(mut self, size: i32) -> Self {
        self.size = size;
        self
    }

    pub fn publisher(mut self, publisher: impl Into<String>) -> Self {
        self.publisher = Some(publisher.into());
        self
    }

    pub fn start_pub_date(mut self, date: NaiveDate) -> Self {
        self.start_pub_date = Some(date);
        self
    }

    pub fn end_pub_date(mut self, date: NaiveDate) -> Self {
        self.end_pub_date = Some(date);
        self
    }

    pub fn build(self) -> Result<Request, &'static str> {
        let publisher = self.publisher.ok_or("출판사는 반드시 입력 되어야 합니다.")?;
        let start_pub_date = self.start_pub_date.ok_or("출판 시작일은 반드시 입력 되어야 합니다.")?;
        let end_pub_date = self.end_pub_date.ok_or("출판 종료일은 반드시 입력 되어야 합니다.")?;

        Ok(Request {
            page: self.page,
            size: self.size,
            publisher,
            start_pub_date,
            end_pub_date,
        })
    }
}

/// 국립중앙도서관 API 클라이언트
pub struct Client {
    /// API 인증 키
    key: String
}

impl Client {
    pub fn new(key: String) -> Client {
        Client { key }
    }

    /// 도서 정보 검색 메서드 - 요청 매개변수를 받아 API를 호출하고 응답을 파싱하여 반환
    pub fn get_books(&self, request: Request) -> Result<Response, ClientError> {
        let url = self.build_search_url(&request)?;
        let response = reqwest::blocking::get(url)
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;
        let response_text = response.text()
            .map_err(|e| ClientError::ResponseTextExtractionFailed(e.to_string()))?;
        let parsed_response: Response = serde_json::from_str(&response_text)
            .map_err(|e| ClientError::ResponseParseFailed(e.to_string()))?;
        Ok(parsed_response)
    }

    fn build_search_url(&self, request: &Request) -> Result<reqwest::Url, ClientError> {
        let from = request.start_pub_date.format("%Y%m%d").to_string();
        let to = request.end_pub_date.format("%Y%m%d").to_string();

        // URL 생성
        let mut url = reqwest::Url::parse(ISBN_SEARCH_ENDPOINT)
            .map_err(|_| ClientError::InvalidBaseUrl)?;

        // 쿼리 파라미터 추가
        url.query_pairs_mut()
            .append_pair("cert_key", &self.key)
            .append_pair("start_publish_date", &from)
            .append_pair("end_publish_date", &to)
            .append_pair("publisher", &request.publisher)
            .append_pair("result_style", "json")
            .append_pair("page_no", &request.page.to_string())
            .append_pair("page_size", &request.size.to_string())
            .append_pair("sort", "INDEX_PUBLISHER")
            .append_pair("order_by", "ASC");

        Ok(url)
    }
}