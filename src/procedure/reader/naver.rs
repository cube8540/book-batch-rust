use crate::item::{Book, BookRepository};
use crate::procedure::reader::Reader;
use crate::procedure::Parameter;
use crate::provider;
use crate::provider::api::{naver, Client};

pub struct NaverReader<R>
where
    R: BookRepository,
{
    client: naver::Client,
    repository: R,
}

pub fn new<R>(client: naver::Client, repository: R) -> NaverReader<R>
where
    R: BookRepository,
{
    NaverReader { client, repository }
}

impl <R> Reader for NaverReader<R>
where
    R: BookRepository,
{
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let (from, to) = (parameter.from().as_ref().unwrap(), parameter.to().as_ref().unwrap());
        self.repository.find_by_pub_between(from, to).into_iter()
            .flat_map(|book| {
                let request = provider::api::Request::builder()
                    .query(book.isbn().to_owned())
                    .build().unwrap();
                self.client.get_books(&request).unwrap().books
                    .into_iter()
                    .map(|b| b.build().unwrap())
            })
            .collect()
    }
}