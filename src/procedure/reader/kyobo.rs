use crate::book::{repository, Book};
use crate::procedure::reader::Reader;
use crate::procedure::Parameter;
use crate::provider::html::{kyobo, Client};
use tracing::error;

pub struct KyoboReader<R, P>
where
    R: repository::BookRepository,
    P: kyobo::LoginProvider
{
    client: kyobo::Client<P>,
    repository: R
}

impl <R, P> KyoboReader<R, P>
where
    R: repository::BookRepository,
    P: kyobo::LoginProvider
{
    pub fn new(client: kyobo::Client<P>, repository: R) -> Self {
        Self {
            client,
            repository
        }
    }

}

impl<R, P> Reader for KyoboReader<R, P>
where
    R: repository::BookRepository,
    P: kyobo::LoginProvider
{
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let (from, to) = (parameter.from().as_ref().unwrap(), parameter.to().as_ref().unwrap());
        let books = self.repository.find_by_pub_date(from, to);
        books.iter()
            .filter(|book| book.actual_pub_date.is_some())
            .map(|book| {
                self.client.get(book.isbn.as_str())
                    .map(|parsed_book| Some(parsed_book))
                    .unwrap_or_else(|e| {
                        error!("ISBN: {}, ERROR: {:?}", book.isbn, e);
                        None
                    })
            })
            .filter(|book| book.is_some())
            .map(|book| book.unwrap())
            .collect::<Vec<Book>>()
    }
}