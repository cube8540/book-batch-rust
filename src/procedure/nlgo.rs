use crate::book::{Book, Site};
use crate::procedure::Reader;
use crate::provider;
use crate::provider::api::Client;
use chrono::NaiveDate;

pub struct NlgoReader {
    client: provider::api::nlgo::Client,

    from: NaiveDate,
    to: NaiveDate,
}

impl NlgoReader {
    pub fn new(
        client: provider::api::nlgo::Client,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Self {
        Self { client, from, to }
    }
}

impl Reader for NlgoReader {

    fn read(&self, keyword: &str) -> Vec<Book> {
        let mut v = Vec::new();
        let mut page = 1;
        loop {
            let request = provider::api::Request {
                page,
                size: 500,
                query: keyword.to_string(),
                start_date: Some(self.from),
                end_date: Some(self.to),
            };
            let response = self.client.get_books(&request).unwrap(); // TODO 에러 처리 해야함
            if !response.books.is_empty() {
                response.books.into_iter().for_each(|b| v.push(b));
                page += 1;
            } else {
                break v;
            }
        }
    }

    fn site(&self) -> Site {
        provider::api::nlgo::SITE.to_string()
    }
}