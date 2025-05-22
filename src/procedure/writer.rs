use crate::item::{Book, BookRepository};
use std::collections::HashMap;

const WRITE_SIZE: usize = 100;

pub trait Writer {
    fn write(&self, books: &[Book]) -> Vec<Book>;
}

pub struct NewBookOnlyWriter<R>
where
    R: BookRepository
{
    repository: R,
}

impl <R: BookRepository> NewBookOnlyWriter<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl <R: BookRepository> Writer for NewBookOnlyWriter<R> {
    fn write(&self, books: &[Book]) -> Vec<Book> {
        let exists_books = get_target_books(&self.repository, books);
        let not_exists_books: Vec<&Book> = books.iter()
            .filter(|b| !exists_books.contains_key(b.isbn()))
            .collect();

        let chunks = not_exists_books.chunks(WRITE_SIZE);
        chunks.into_iter()
            .flat_map(|books| self.repository.save_books(books))
            .collect()
    }
}

pub struct UpsertBookWriter<R>
where
    R: BookRepository
{
    repository: R
}

impl <R: BookRepository> UpsertBookWriter<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl <R: BookRepository> Writer for UpsertBookWriter<R> {
    fn write(&self, books: &[Book]) -> Vec<Book> {
        let mut exists = get_target_books(&self.repository, books);

        let mut new_books: Vec<&Book> = vec![];
        let mut update_books: Vec<Book> = vec![];

        for book in books {
            if let Some(mut ext) = exists.remove(book.isbn()) {
                ext.merge(book);
                update_books.push(ext);
            } else {
                new_books.push(book);
            }
        }

        new_books.chunks(WRITE_SIZE).into_iter()
            .for_each(|books| {
                self.repository.save_books(books);
            });
        update_books.iter().for_each(|book| {
            self.repository.update_book(book);
        });

        self.repository.find_by_isbn(books.iter().map(|b| b.isbn()).collect::<Vec<&str>>().as_slice())
    }
}

fn get_target_books<R>(repository: &R, target: &[Book]) -> HashMap<String, Book>
where
    R: BookRepository
{
    let isbn: Vec<&str> = target.iter().map(|b| b.isbn()).collect();
    repository.find_by_isbn(&isbn).into_iter()
        .map(|b| (b.isbn().to_owned(), b))
        .collect()
}
