use crate::book::repository::BookRepository;
use crate::book::Book;
use std::collections::HashMap;

pub trait Writer {
    fn write(&self, books: &[&Book]) -> Vec<Book>;
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
    fn write(&self, books: &[&Book]) -> Vec<Book> {
        let exists = get_target_books(&self.repository, books);

        let filtered_books = books.iter()
            .filter(|b| !exists.contains_key(&b.isbn))
            .cloned();

        self.repository.new_books(filtered_books, true)
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
    fn write(&self, books: &[&Book]) -> Vec<Book> {
        let mut exists = get_target_books(&self.repository, &books);

        let mut new_books: Vec<&Book> = vec![];
        let mut update_books: Vec<Book> = vec![];

        for book in books {
            if let Some(mut ext) = exists.remove(&book.isbn) {
                ext.merge(book);
                update_books.push(ext);
            } else {
                new_books.push(book);
            }
        }

        self.repository.new_books(new_books, true);
        self.repository.update_books(update_books.iter(), true);

        self.repository.find_by_isbn(books.iter().map(|b| b.isbn.as_str()))
    }
}

fn get_target_books<R>(repository: &R, target: &[&Book]) -> HashMap<String, Book>
where
    R: BookRepository
{
    repository.find_by_isbn(target.iter().map(|b| b.isbn.as_str())).into_iter()
        .map(|b| (b.isbn.clone(), b))
        .collect::<HashMap<String, Book>>()
}