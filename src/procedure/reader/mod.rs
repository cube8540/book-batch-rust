mod aladin;
mod nlgo;

use crate::book::Book;
use crate::procedure;

use procedure::Parameter;

pub trait Reader {
    fn read_books(&self, parameter: &Parameter) -> Vec<Book>;
}

pub trait FromPublisher: Reader {

    fn read_books(&self, parameter: &Parameter) -> Vec<Book> {
        let publisher = parameter.publisher.unwrap();
        if let Some(keywords) = publisher.keywords.get(&parameter.site) {
            keywords.iter()
                .flat_map(|keyword| {
                    let mut books = self.read(keyword);
                    books.iter_mut().for_each(|b| b.publisher_id = publisher.id);
                    books
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn read(&self, keyword: &str) -> Vec<Book>;
}