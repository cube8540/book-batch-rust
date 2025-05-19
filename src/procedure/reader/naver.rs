use crate::book::{repository, Book};
use crate::procedure::reader::Reader;
use crate::procedure::Parameter;
use crate::provider;
use crate::provider::api::{naver, Client};

pub struct NaverReader<R>
where
    R: repository::BookRepository,
{
    client: naver::Client,
    repository: R,
}

pub fn new<R>(client: naver::Client, repository: R) -> NaverReader<R>
where
    R: repository::BookRepository,
{
    NaverReader { client, repository }
}

impl <R> Reader for NaverReader<R>
where
    R: repository::BookRepository,
{
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let (from, to) = (parameter.from().as_ref().unwrap(), parameter.to().as_ref().unwrap());
        let books = self.repository.find_by_pub_date(from, to);
        books.iter()
            .flat_map(|b| {
                let request = provider::api::Request {
                    page: 0,
                    size: 0,
                    query: b.isbn().to_owned(),
                    start_date: None,
                    end_date: None,
                };
                self.client.get_books(&request).unwrap().books
            })
            .collect()
    }
}