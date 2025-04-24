use crate::provider::error::ClientError;
use crate::{book, provider};
use serde::Deserialize;
use serde_with::serde_as;
use std::collections::HashMap;

/// 국립중앙도서관 ISBN 도서정보 검색 API 엔드포인트 URL
const ISBN_SEARCH_ENDPOINT: &'static str = "https://www.nl.go.kr/seoji/SearchApi.do";
/// API 요청 시 기본 타임아웃 시간(초)
const DEFAULT_TIMEOUT_SECONDS: i64 = 10;
/// 검색 결과 기본 페이지 번호
const DEFAULT_PAGE: i32 = 1;
/// 페이지당 기본 결과 수
const DEFAULT_SIZE: i32 = 50;

pub const SITE: &'static str = "nlgo";

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

impl Doc {
    fn to_map(&self) -> HashMap<String, String> {
        HashMap::from([
            ("title".to_string(), self.title.clone()),
            ("ea_isbn".to_string(), self.ea_isbn.clone()),
            ("set_isbn".to_string(), self.set_isbn.clone()),
            ("ea_add_code".to_string(), self.ea_add_code.clone()),
            ("set_add_code".to_string(), self.set_add_code.clone()),
            ("series_no".to_string(), self.series_no.clone()),
            ("set_expression".to_string(), self.set_expression.clone()),
            ("subject".to_string(), self.subject.clone()),
            ("publisher".to_string(), self.publisher.clone()),
            ("author".to_string(), self.author.clone()),
            ("real_publish_date".to_string(), self.real_publish_date.clone()),
            ("publish_predate".to_string(), self.publish_predate.clone()),
            ("update_date".to_string(), self.update_date.clone()),
        ])
    }
}

/// API 응답 구조체로 검색 결과 메타데이터와 도서 정보 목록 포함
#[serde_as]
#[derive(Deserialize)]
pub struct Response {
    /// 검색된 총 도서 수
    #[serde(rename = "TOTAL_COUNT")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub total_count: i32,

    /// 현재 페이지 번호
    #[serde(rename = "PAGE_NO")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub page_no: i32,

    /// 검색된 도서 목록
    pub docs: Vec<Doc>,
}

/// 국립중앙도서관 API 클라이언트
pub struct Client {
    /// API 인증 키
    key: String
}

impl Client {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    fn build_search_url(&self, request: &provider::Request) -> Result<reqwest::Url, ClientError> {
        let from = if let Some(date) = request.start_date {
            date.format("%Y%m%d").to_string()
        } else {
            return Err(ClientError::MissingRequiredParameter("시작일은 반드시 입력 되어야 합니다.".to_string()))
        };
        let to = if let Some(date) = request.end_date {
            date.format("%Y%m%d").to_string()
        } else {
            return Err(ClientError::MissingRequiredParameter("종료일은 반드시 입력 되어야 합니다.".to_string()))
        };

        // URL 생성
        let mut url = reqwest::Url::parse(ISBN_SEARCH_ENDPOINT)
            .map_err(|_| ClientError::InvalidBaseUrl)?;

        // 쿼리 파라미터 추가
        url.query_pairs_mut()
            .append_pair("cert_key", &self.key)
            .append_pair("start_publish_date", &from)
            .append_pair("end_publish_date", &to)
            .append_pair("publisher", &request.query)
            .append_pair("result_style", "json")
            .append_pair("page_no", &request.page.to_string())
            .append_pair("page_size", &request.size.to_string())
            .append_pair("sort", "INDEX_PUBLISHER")
            .append_pair("order_by", "ASC");

        Ok(url)
    }
}

impl provider::Client for Client {
    fn get_books(&self, request: &provider::Request) -> Result<provider::Response, ClientError> {
        let url = self.build_search_url(&request)?;
        let response = reqwest::blocking::get(url)
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;
        let response_text = response.text()
            .map_err(|e| ClientError::ResponseTextExtractionFailed(e.to_string()))?;
        let parsed_response: Response = serde_json::from_str(&response_text)
            .map_err(|e| ClientError::ResponseParseFailed(e.to_string()))?;

        let books = parsed_response.docs.iter()
            .map(|doc| convert_doc_to_book(doc));

        Ok(provider::Response {
            total_count: parsed_response.total_count,
            page_no: parsed_response.page_no,
            site: SITE.to_string(),
            books: books.collect(),
        })
    }
}

fn convert_doc_to_book(doc: &Doc) -> book::Book {
    let scheduled_pub_date = if doc.publish_predate != "" {
        chrono::NaiveDate::parse_from_str(&doc.publish_predate, "%Y%m%d").ok()
    } else {
        None
    };
    let actual_pub_date = if doc.real_publish_date != "" {
        chrono::NaiveDate::parse_from_str(&doc.real_publish_date, "%Y%m%d").ok()
    } else {
        None
    };
    book::Book {
        id: 0,
        isbn: doc.ea_isbn.clone(),
        publisher_id: 0,
        title: doc.title.clone(),
        scheduled_pub_date,
        actual_pub_date,
        origin_data: HashMap::from([(SITE.to_string(), doc.to_map())]),
    }
}