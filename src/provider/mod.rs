use crate::book;
use crate::provider::error::ClientError;
use chrono::NaiveDate;

pub mod error;
pub mod nlgo;
pub mod aladin;

const DEFAULT_PAGE: i32 = 1;
const DEFAULT_SIZE: i32 = 100;

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