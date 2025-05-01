use crate::book::{Book, Node, Original, Publisher, Site};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

pub mod diesel;

#[derive(Debug)]
pub enum SQLError {
    QueryExecuteError(String),
}

impl fmt::Display for SQLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SQLError::QueryExecuteError(msg) => write!(f, "{}", msg),
        }
    }
}

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

    fn new_books<'book, I>(&self, books: I, with_origin: bool) -> Result<Vec<Book>, SQLError>
    where
        I: IntoIterator<Item=&'book Book>;

    fn new_origin_data<'book, I>(&self, origins: I) -> Result<usize, SQLError>
    where
        I: IntoIterator<Item=(u64, &'book HashMap<Site, Original>)>;

    fn update_book(&self, book: &Book, with_origin: bool) -> Result<usize, SQLError>;

    fn delete_origin_data(&self, id: u64, site: &Site) -> Result<usize, SQLError>;
}

pub trait BookOriginFilterRepository {
    fn get_root_filters(&self) -> Vec<Node>;
}