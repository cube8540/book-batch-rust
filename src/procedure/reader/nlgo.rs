use crate::book::{Book, Site};
use crate::procedure::reader::{FromPublisher, Reader};
use crate::procedure::Parameter;
use crate::provider;
use crate::provider::api::Client;
use chrono::NaiveDate;
use provider::api::nlgo;

pub struct NlgoReader {
    client: nlgo::Client,

    from: NaiveDate,
    to: NaiveDate,
}

pub fn new(client: nlgo::Client, from: NaiveDate, to: NaiveDate) -> NlgoReader {
    NlgoReader { client, from, to }
}

impl Reader for NlgoReader {
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        <Self as FromPublisher>::read_books(self, parameter)
    }
}

impl FromPublisher for NlgoReader {
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
        nlgo::SITE.to_owned()
    }
}