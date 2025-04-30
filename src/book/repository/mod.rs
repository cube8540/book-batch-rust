use crate::book::{Book, Node, Original, Publisher, Site};
use std::collections::HashMap;

pub mod diesel;


pub trait PublisherRepository {
    fn get_all(&self) -> Vec<Publisher>;
}

pub trait BookRepository {

    fn find_by_isbn<'book, I>(&self, isbn: I) -> Vec<Book>
    where
        I: Iterator<Item=&'book str>;

    fn find_origin_by_id<I>(&self, id: I) -> HashMap<u64, HashMap<Site, Original>>
    where
        I: Iterator<Item=u64>;

    fn new_books<'book, I>(&self, books: I, with_origin: bool) -> Vec<Book>
    where
        I: IntoIterator<Item=&'book Book>;

    fn new_origin_data<'book, I>(&self, origins: I) -> usize
    where
        I: IntoIterator<Item=(u64, &'book HashMap<Site, Original>)>;

    fn update_books<'book, I>(&self, books: I, with_origin: bool) -> usize
    where
        I: IntoIterator<Item=&'book Book>;

    fn delete_origin_data(&self, id: u64, site: &Site) -> usize;
}

pub trait BookOriginFilterRepository {
    fn get_root_filters(&self) -> HashMap<Site, Node>;
}