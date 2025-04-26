use crate::book::{Book, Publisher, Site};
use crate::procedure::Reader;
use crate::provider;
use crate::provider::Client;
use chrono::NaiveDate;

pub struct NlgoReader {
    client: provider::nlgo::Client,

    from: NaiveDate,
    to: NaiveDate,
}

impl NlgoReader {
    pub fn new(
        client: provider::nlgo::Client,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Self {
        Self { client, from, to }
    }
}

impl Reader for NlgoReader {

    fn read(&self, keyword: &str) -> Vec<Box<Book>> {
        let mut v = Vec::new();
        let mut page = 1;
        loop {
            let request = provider::Request {
                page,
                size: 500,
                query: keyword.to_string(),
                start_date: Some(self.from),
                end_date: Some(self.to),
            };
            let response = self.client.get_books(&request).unwrap(); // TODO 에러 처리 해야함
            if !response.books.is_empty() {
                response.books.iter().for_each(|book| v.push(book.clone()));
                page += 1;
            } else {
                break v;
            }
        }
    }

    fn site(&self) -> Site {
        provider::nlgo::SITE.to_string()
    }
}