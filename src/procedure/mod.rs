pub mod nlgo;
pub mod aladin;

use crate::book;
use crate::book::{BookOriginFilterRepository, BookRepository};
use book::Book;
use std::collections::HashMap;

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
    fn write(&self, books: Vec<Book>) -> Vec<Book>; // TODO 에러 처리
}

pub struct OnlyInsertWriter<R: BookRepository> {
    repository: R
}

impl <R: BookRepository> Writer for OnlyInsertWriter<R> {
    fn write(&self, books: Vec<Book>) -> Vec<Book> {
        let exists = get_target_books(&self.repository, &books);

        let new_books = books.into_iter()
            .filter(|b| !exists.contains_key(&b.isbn))
            .collect::<Vec<Book>>();

        self.repository.new_books(new_books)
    }
}

pub struct UpsertWriter<R: BookRepository> {
    repository: R
}

impl <R: BookRepository> Writer for UpsertWriter<R> {
    fn write(&self, books: Vec<Book>) -> Vec<Book> {
        let mut exists = get_target_books(&self.repository, &books);

        let mut new_books = vec![];
        let mut update_books = vec![];

        books.into_iter().for_each(|book| {
            if let Some(ext) = exists.remove(&book.isbn) {
                update_books.push(ext.merge(book));
            } else {
                new_books.push(book);
            }
        });

        let mut result = vec![];
        self.repository.new_books(new_books).into_iter().for_each(|b| result.push(b));
        self.repository.update_books(update_books).into_iter().for_each(|b| result.push(b));
        result
    }
}

fn get_target_books<R: BookRepository>(repository: &R, target: &Vec<Book>) -> HashMap<String, Book> {
    let isbn = target.iter().map(|b| b.isbn.as_str()).collect();

    repository.get_by_isbn(&isbn).into_iter()
        .map(|b| (b.isbn.clone(), b))
        .collect::<HashMap<String, Book>>()
}