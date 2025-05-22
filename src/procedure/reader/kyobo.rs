use crate::item::{Book, BookRepository};
use crate::procedure::reader::Reader;
use crate::procedure::Parameter;
use crate::provider::html::{kyobo, Client};
use tracing::error;

pub struct KyoboReader<R, P>
where
    R: BookRepository,
    P: kyobo::LoginProvider
{
    client: kyobo::Client<P>,
    repository: R
}

impl <R, P> KyoboReader<R, P>
where
    R: BookRepository,
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
    R: BookRepository,
    P: kyobo::LoginProvider
{
    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let (from, to) = (parameter.from().as_ref().unwrap(), parameter.to().as_ref().unwrap());
        let books = self.repository.find_by_pub_between(from, to);
        books.iter()
            .filter(|book| book.actual_pub_date().is_some())
            .filter_map(|book| {
                self.client.get(book.isbn())
                    .map(|parsed_book| parsed_book.build().unwrap())
                    .inspect_err(|err| error!("ISBN: {}, ERROR: {:?}", book.isbn(), err))
                    .ok()
            })
            .collect()
    }
}