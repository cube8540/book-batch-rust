pub mod nlgo;
pub mod aladin;

use crate::book;
use crate::book::BookOriginFilterRepository;
use book::Book;

pub trait Reader {
    fn get_books(&self, publisher: &book::Publisher) -> Vec<Book> {
        publisher.keywords(self.site()).iter()
            .flat_map(|keyword| {
                let mut books = self.read(keyword);
                books.iter_mut().for_each(|b| b.publisher_id = publisher.id());
                books
            })
            .collect()
    }
    
    fn read(&self, keyword: &str) -> Vec<Book>;
    
    fn site(&self) -> book::Site;
}

pub trait Filter {
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book>;
}

pub struct OriginDataFilter {
    repository: Box<dyn BookOriginFilterRepository>,
    site: book::Site,
}

impl OriginDataFilter {
    pub fn new(repository: Box<dyn BookOriginFilterRepository>, site: book::Site) -> Self {
        Self {
            repository,
            site,
        }
    }
}

impl Filter for OriginDataFilter {
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book> {
        let filter_map = self.repository.get_root_filters();

        match filter_map.get(&self.site) {
            Some(filter) => books.into_iter()
                .filter(|book| {
                    book.origin_data
                        .get(&self.site)
                        .map_or(true, |origin| filter.borrow().validate(origin))
                })
                .collect(),
            None => books
        }
    }
}

pub trait Writer {
    fn write(&self, books: &Vec<Book>); // TODO 에러 처리
}