pub mod nlgo;
pub mod aladin;

use crate::book;
use crate::book::{BookOriginFilterRepository, BookRepository, Publisher, Site};
use book::Book;
use std::collections::HashMap;

pub struct Parameter<'job> {
    pub isbn: Option<&'job str>,
    pub publisher: Option<&'job Publisher>,
}

pub trait Reader {
    fn get_books(&self, parameter: &Parameter) -> Vec<Book>;
}

pub fn read_by_publisher<F>(site: Site, publisher: &Publisher, f: F) -> Vec<Book>
where
    F: Fn(&str) -> Vec<Book> {
    publisher.keywords(site).iter()
        .flat_map(|keyword| {
            let mut books = f(keyword);
            books.iter_mut().for_each(|b| b.publisher_id = publisher.id);
            books
        })
        .collect()
}

pub trait Filter {
    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where 'job: 'input;
}

pub struct FilterChain {
    next: Vec<Box<dyn Filter>>
}

impl FilterChain {
    pub fn new() -> Self {
        Self {
            next: vec![]
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter>) {
        self.next.push(filter);
    }
}

impl Filter for FilterChain {
    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where
        'job: 'input {
        self.next.iter()
            .fold(books.to_vec(), |books, filter| filter.do_filter(&books))
    }
}

pub struct EmptyIsbnFilter;

impl EmptyIsbnFilter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Filter for EmptyIsbnFilter {

    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where
        'job: 'input{
        books.iter()
            .filter(|b| !b.isbn.is_empty())
            .copied()
            .collect()
    }
}

pub struct OriginDataFilter<R: BookOriginFilterRepository> {
    repository: R,
    site: Site,
}

impl <R: BookOriginFilterRepository> OriginDataFilter<R> {
    pub fn new(repository: R, site: Site) -> Self {
        Self {
            repository,
            site,
        }
    }
}

impl <R: BookOriginFilterRepository> Filter for OriginDataFilter<R> {
    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where
        'job: 'input {
        let filter_map = self.repository.get_root_filters();

        if let Some(filter) = filter_map.get(&self.site) {
            books.iter()
                .filter(|b| {
                    b.origin_data.get(&self.site)
                        .map_or(false, |o| filter.borrow().validate(o))
                })
                .copied()
                .collect()
        } else {
            books.iter().copied().collect()
        }
    }
}

pub trait Writer {
    fn write(&self, books: &[&Book]) -> Vec<Book>;
}

pub struct OnlyInsertWriter<R: BookRepository> {
    repository: R
}

impl <R: BookRepository> OnlyInsertWriter<R> {

    pub fn new(repository: R) -> Self {
        Self {
            repository
        }
    }
}

impl <R: BookRepository> Writer for OnlyInsertWriter<R> {
    fn write(&self, books: &[&Book]) -> Vec<Book> {
        let exists = get_target_books(&self.repository, &books);

        let new_books: Vec<&Book> = books.iter()
            .filter(|b| !exists.contains_key(&b.isbn))
            .copied()
            .collect();

        self.repository.new_books(&new_books)
    }
}

pub struct UpsertWriter<R: BookRepository> {
    repository: R
}

impl <R: BookRepository> UpsertWriter<R> {

    pub fn new(repository: R) -> Self {
        Self {
            repository
        }
    }
}

impl <R: BookRepository> Writer for UpsertWriter<R> {
    fn write(&self, books: &[&Book]) -> Vec<Book> {
        let mut exists = get_target_books(&self.repository, &books);

        let mut new_books: Vec<&Book> = vec![];
        let mut update_books: Vec<Book> = vec![];

        books.iter().for_each(|book| {
            if let Some(mut ext) = exists.remove(&book.isbn) {
                ext.merge(book);
                update_books.push(ext);
            } else {
                new_books.push(book);
            }
        });

        let new_books = self.repository.new_books(&new_books);
        let update_books = self.repository.update_books(&update_books.iter().collect::<Vec<&Book>>());

        new_books.into_iter().chain(update_books).collect()
    }
}

fn get_target_books<R: BookRepository>(repository: &R, target: &[&Book]) -> HashMap<String, Book> {
    let isbn = target.iter()
        .map(|b| b.isbn.as_str())
        .collect::<Vec<&str>>();

    repository.get_book_only_by_isbn(&isbn).into_iter()
        .map(|b| (b.isbn.clone(), b))
        .collect::<HashMap<String, Book>>()
}