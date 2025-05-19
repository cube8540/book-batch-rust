use crate::book::repository::BookRepository;
use crate::book::Book;
use std::collections::HashMap;
use tracing::error;

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
        let exists = get_target_books(&self.repository, books);

        let filtered_books: Vec<&Book> = books.iter()
            .filter(|b| !exists.contains_key(b.isbn()))
            .collect();

        let chunks = filtered_books.chunks(WRITE_SIZE);
        chunks.into_iter()
            .flat_map(|books| {
                let result = self.repository.new_books(books.iter().cloned(), true);
                if let Ok(books) = result {
                    books
                } else {
                    let isbn: Vec<&str> = books.iter().map(|b| b.isbn()).collect();
                    error!("도서 저장 중 에러가 발생 했습니다 {:?} (ISBN => {:?})", result.unwrap_err(), isbn);
                    vec![]
                }
            })
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

        new_books.chunks(WRITE_SIZE).into_iter().for_each(|books| {
            if let Err(err) = self.repository.new_books(books.iter().cloned(), true) {
                let isbn: Vec<&str> = books.iter().map(|b| b.isbn()).collect();
                error!("도서 저장 중 에러가 발생 했습니다 {:?} (ISBN => {:?})", err, isbn);
            }
        });
        update_books.iter().for_each(|book| {
            if let Err(err) = self.repository.update_book(book, true) {
                error!("도서 저장 중 에러가 발생 했습니다. {:?} (ISBN => {})", err, book.isbn());
            }
        });

        self.repository.find_by_isbn(books.iter().map(|b| b.isbn()))
    }
}

fn get_target_books<R>(repository: &R, target: &[Book]) -> HashMap<String, Book>
where
    R: BookRepository
{
    repository.find_by_isbn(target.iter().map(|b| b.isbn()))
        .into_iter()
        .map(|b| (b.isbn().to_owned(), b))
        .collect::<HashMap<String, Book>>()
}
