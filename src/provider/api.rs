use crate::item::{BookBuilder, Site};
use chrono::NaiveDate;

pub mod nlgo;
pub mod aladin;
pub mod naver;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    MissingRequiredParameter(String), // 필수 매개변수가 누락됨
    InvalidBaseUrl,
    RequestFailed(String),
    ResponseTextExtractionFailed(String),
    ResponseParseFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestError {
    InvalidParameter(String),         // 유효하지 않은 매개변수
}

#[derive(Debug)]
pub struct Request {
    page: i32,
    size: i32,
    query: String,
    start_date:Option<NaiveDate>,
    end_date:Option<NaiveDate>,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::new()
    }

    pub fn page(&self) -> i32 {
        self.page
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn start_date(&self) -> Option<NaiveDate> {
        self.start_date
    }

    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end_date
    }
}

#[derive(Default)]
pub struct RequestBuilder {
    page: Option<i32>,
    size: Option<i32>,
    query: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder::default()
    }

    pub fn page(mut self, page: i32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn size(mut self, size: i32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn query<S: Into<String>>(mut self, query: S) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn start_date(mut self, start_date: NaiveDate) -> Self {
        self.start_date = Some(start_date);
        self
    }

    pub fn end_date(mut self, end_date: NaiveDate) -> Self {
        self.end_date = Some(end_date);
        self
    }

    pub fn build(self) -> Result<Request, RequestError> {
        let query = self.query.ok_or_else(|| 
            RequestError::InvalidParameter("query is required".to_string()))?;

        let page = self.page.unwrap_or(0);
        if page < 0 {
            return Err(RequestError::InvalidParameter("page must be greater than or equal to 0".to_string()));
        }

        let size = self.size.unwrap_or(0);
        if size < 0 {
            return Err(RequestError::InvalidParameter("size must be greater than or equal to 0".to_string()));
        }

        Ok(Request {
            page,
            size,
            query,
            start_date: self.start_date,
            end_date: self.end_date,
        })
    }
}

#[derive(Debug)]
pub struct Response {
    pub total_count: i32,
    pub page_no: i32,
    pub site: Site,
    pub books: Vec<BookBuilder>,
}

impl Response {
    pub fn empty(site: Site) -> Self {
        Response {
            total_count: 0,
            page_no: 0,
            site,
            books: Vec::new(),
        }
    }
}

pub trait Client {
    fn get_books(&self, request: &Request) -> Result<Response, ClientError>;
}