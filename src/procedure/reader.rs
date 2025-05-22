pub mod aladin;
pub mod nlgo;
pub mod naver;
pub mod kyobo;

use crate::item::{Book, BookBuilder, Site};
use crate::procedure;

use procedure::Parameter;

pub trait Reader {
    fn read_books(&self, parameter: &Parameter) -> Vec<Book>;
}

pub trait FromPublisher: Reader {

    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let publisher = parameter.publisher().as_ref().unwrap();

        publisher.keywords().get(&self.site())
            .map(|keywords| {
                keywords.iter()
                    .flat_map(|keyword| self.read(keyword, parameter))
                    .map(|b| b.publisher_id(publisher.id()).build().unwrap())
                    .collect()
            })
            .unwrap_or_else(|| vec![])
    }

    fn read(&self, keyword: &str, parameter: &Parameter) -> Vec<BookBuilder>;

    fn site(&self) -> Site;
}