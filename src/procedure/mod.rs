pub mod nlgo;
pub mod aladin;

use crate::book;
use book::Book;

pub trait Reader {
    fn get_books(&self, publisher: &book::Publisher) -> Vec<Box<Book>> {
        publisher.keywords(self.site()).iter()
            .flat_map(|keyword| {
                let mut books = self.read(keyword);
                books.iter_mut().for_each(|b| b.publisher_id = publisher.id());
                books
            })
            .collect()
    }
    
    fn read(&self, keyword: &str) -> Vec<Box<Book>>;
    
    fn site(&self) -> book::Site;
}

pub trait Filter {
    fn filter(&self, books: &Vec<Box<Book>>) -> Vec<Box<Book>>;
}

pub trait Writer {
    fn write(&self, books: &Vec<Box<Book>>); // TODO 에러 처리
}