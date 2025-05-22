use crate::item::{Book, BookBuilder, Site};
use crate::procedure::reader::{FromPublisher, Reader};
use crate::procedure::Parameter;
use crate::provider;
use crate::provider::api::Client;
use provider::api::aladin;

const PAGE_SIZE: i32 = 50;
const MAX_RESULTS: usize = 200; // 알라딘 API 최대 조회 제한

pub struct AladinReader {
    client: aladin::Client,
}

pub fn new(client: aladin::Client) -> AladinReader {
    AladinReader { client }
}

impl Reader for AladinReader {
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        <Self as FromPublisher>::read_books(self, parameter)
    }
}

impl FromPublisher for AladinReader {
    fn read(&self, keyword: &str, _: &Parameter) -> Vec<BookBuilder> {
        let mut v = Vec::new();
        let mut total_fetched = 0;
        let mut page = 1;
        loop {
            let request = provider::api::Request {
                page,
                size: PAGE_SIZE,
                query: keyword.to_string(),
                start_date: None,
                end_date: None,
            };
            let current_response = self.client.get_books(&request).unwrap(); // TODO: 에러 처리 필요
            if !current_response.books.is_empty() {
                total_fetched += current_response.books.len();
                current_response.books.into_iter().for_each(|b| v.push(b));
                page += 1;
                if total_fetched >= MAX_RESULTS {
                    break v;
                }
            } else {
                break v;
            }
        }
    }

    fn site(&self) -> Site {
        Site::Aladin
    }
}