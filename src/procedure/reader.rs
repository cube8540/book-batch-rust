pub mod aladin;
pub mod nlgo;
pub mod naver;
pub mod kyobo;

use crate::book::{Book, Site};
use crate::procedure;

use procedure::Parameter;

pub trait Reader {
    fn read_books(&self, parameter: &Parameter) -> Vec<Book>;
}

pub trait FromPublisher: Reader {

    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let publisher = parameter.publisher().as_ref().unwrap();
        if let Some(keywords) = publisher.keywords.get(&self.site()) {
            keywords.iter()
                .flat_map(|keyword| {
                    let mut books = self.read(keyword, parameter);
                    books.iter_mut().for_each(|b| b.publisher_id = publisher.id);
                    books
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn read(&self, keyword: &str, parameter: &Parameter) -> Vec<Book>;

    fn site(&self) -> Site;
}