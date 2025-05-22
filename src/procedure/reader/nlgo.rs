use crate::item::{Book, BookBuilder, Site};
use crate::procedure::reader::{FromPublisher, Reader};
use crate::procedure::Parameter;
use crate::provider;
use crate::provider::api::Client;
use provider::api::nlgo;

const PAGE_SIZE: i32 = 500;

pub struct NlgoReader {
    client: nlgo::Client,
}

pub fn new(client: nlgo::Client) -> NlgoReader {
    NlgoReader { client }
}

impl Reader for NlgoReader {
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        <Self as FromPublisher>::read_books(self, parameter)
    }
}

impl FromPublisher for NlgoReader {
    fn read(&self, keyword: &str, parameter: &Parameter) -> Vec<BookBuilder> {
        let mut v = Vec::new();
        let mut page = 1;
        loop {
            let request = provider::api::Request::builder()
                .page(page).size(PAGE_SIZE)
                .query(keyword.to_owned())
                .start_date(parameter.from.unwrap())
                .end_date(parameter.to.unwrap())
                .build().unwrap();

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
        Site::NLGO
    }
}