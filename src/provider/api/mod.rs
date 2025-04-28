use crate::book;
use chrono::NaiveDate;

pub mod nlgo;
pub mod aladin;

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
    pub page: i32,
    pub size: i32,
    pub query: String,
    pub start_date:Option<NaiveDate>,
    pub end_date:Option<NaiveDate>,
}

#[derive(Debug)]
pub struct Response {
    pub total_count: i32,
    pub page_no: i32,
    pub site: book::Site,
    pub books: Vec<book::Book>,
}

pub trait Client {
    fn get_books(&self, request: &Request) -> Result<Response, ClientError>;
}