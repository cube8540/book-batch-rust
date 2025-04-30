use std::collections::HashMap;
use crate::book::{Book, Node, Publisher, Site};

mod diesel;


pub trait PublisherRepository {
    fn get_all(&self) -> Vec<Publisher>;
}

pub trait BookRepository {
    fn get_by_isbn<'book, I>(&self, isbn: I) -> Vec<Book>
    where
        I: Iterator<Item = &'book str>;

    fn new_books(&self, books: &[&Book]) -> Vec<Book>;

    fn update_books(&self, books: &[&Book]) -> Vec<Book>;
}

pub trait BookOriginFilterRepository {
    fn get_root_filters(&self) -> HashMap<Site, Node>;
}