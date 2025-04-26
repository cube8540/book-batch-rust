use crate::book::{Book, Site};
use crate::procedure::Reader;
use crate::provider;
use crate::provider::Client;

const PAGE_SIZE: i32 = 50;
const MAX_RESULTS: usize = 200; // 알라딘 API 최대 조회 제한

pub struct AladinReader {
    client: provider::aladin::Client,
}

impl AladinReader {
    pub fn new(
        client: provider::aladin::Client,
    ) -> Self {
        Self { client }
    }

}

impl Reader for AladinReader {
    
    fn read(&self, keyword: &str) -> Vec<Box<Book>> {
        let mut v = Vec::new();

        let mut total_fetched = 0;
        let mut page = 1;
        loop {
            let request = provider::Request {
                page,
                size: PAGE_SIZE,
                query: keyword.to_string(),
                start_date: None,
                end_date: None,
            };
            let current_response = self.client.get_books(&request).unwrap(); // TODO: 에러 처리 필요
            if !current_response.books.is_empty() {
                current_response.books.iter().for_each(|book| v.push(book.clone()));
                page += 1;
                total_fetched += current_response.books.len();
                if total_fetched >= MAX_RESULTS {
                    break v;
                }
            } else {
                break v;
            }
        }
    }

    fn site(&self) -> Site {
        provider::aladin::SITE.to_string()
    }
}