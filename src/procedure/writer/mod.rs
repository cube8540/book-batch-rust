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
        let exists = get_target_books(&self.repository, &books);

        let new_books: Vec<&Book> = books.iter()
            .filter(|b| !exists.contains_key(&b.isbn))
            .copied()
            .collect();

        self.repository.new_books(&new_books)
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
        .map(|b| b.isbn.as_str());

    repository.get_by_isbn(isbn).into_iter()
        .map(|b| (b.isbn.clone(), b))
        .collect::<HashMap<String, Book>>()
}