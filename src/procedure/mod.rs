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

pub trait Filter<'f, 'job: 'f> {
    fn do_filter<'input>(&'f self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where 'job: 'input;
}

pub trait FilterChain<'f, 'job: 'f>: Filter<'f, 'job> {

    fn chain<'input>(&'f self, books: &'input [&'job Book]) -> Vec<&'job Book> {
        if let Some(next) = self.next() {
            let filtered = self.do_filter(books);
            next.do_filter(&filtered)
        } else {
            self.do_filter(books)
        }
    }

    fn next(&'f self) -> &'f Option<Box<dyn Filter<'f, 'job>>>;

    fn add_next(self, filter: Box<dyn Filter<'f, 'job>>) -> Self;
}

pub struct EmptyIsbnFilter<'f, 'job: 'f> {
    next: Option<Box<dyn Filter<'f, 'job>>>
}

impl <'f, 'job: 'f> EmptyIsbnFilter<'_, '_> {
    pub fn new() -> Self {
        Self {
            next: None
        }
    }
}

impl <'f, 'job: 'f> Filter<'f, 'job> for EmptyIsbnFilter<'f, 'job> {

    fn do_filter<'input>(&'f self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where 'job: 'input{
        books.iter()
            .filter(|b| !b.isbn.eq(""))
            .cloned()
            .collect()
    }
}

impl <'f, 'job: 'f> FilterChain<'f, 'job> for EmptyIsbnFilter<'f, 'job> {
    fn next(&'f self) -> &'f Option<Box<dyn Filter<'f, 'job>>> {
        &self.next
    }

    fn add_next(mut self, filter: Box<dyn Filter<'f, 'job>>) -> Self {
        self.next = Some(filter);
        self
    }
}

pub struct OriginDataFilter<'f, 'job: 'f, R: BookOriginFilterRepository> {
    repository: R,
    site: Site,
    next: Option<Box<dyn Filter<'f, 'job>>>,
}

impl <R: BookOriginFilterRepository> OriginDataFilter<'_, '_, R> {
    pub fn new(repository: R, site: Site) -> Self {
        Self {
            repository,
            site,
            next: None,
        }
    }
}

impl <'f, 'job: 'f, R: BookOriginFilterRepository> Filter<'f, 'job> for OriginDataFilter<'f, 'job, R> {
    fn do_filter<'input>(&'f self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where 'job: 'input {
        let filter_map = self.repository.get_root_filters();

        if let Some(filter) = filter_map.get(&self.site) {
            books.iter()
                .filter(|b| {
                    b.origin_data.get(&self.site)
                        .map_or(false, |o| filter.borrow().validate(o))
                })
                .cloned()
                .collect()
        } else {
            books.iter().cloned().collect()
        }
    }
}

impl <'f, 'job: 'f, R: BookOriginFilterRepository> FilterChain<'f, 'job> for OriginDataFilter<'f, 'job, R> {
    fn next(&'f self) -> &'f Option<Box<dyn Filter<'f, 'job>>> {
        &self.next
    }

    fn add_next(mut self, filter: Box<dyn Filter<'f, 'job>>) -> Self {
        self.next = Some(filter);
        self
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

        let new_books = books.iter()
            .filter(|b| !exists.contains_key(&b.isbn))
            .cloned()
            .collect::<Vec<&Book>>();

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

    repository.get_by_isbn(&isbn).into_iter()
        .map(|b| (b.isbn.clone(), b))
        .collect::<HashMap<String, Book>>()
}