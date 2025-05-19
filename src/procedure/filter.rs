use crate::book::repository::BookOriginFilterRepository;
use crate::book::Book;

pub trait Filter {
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book>;
}

pub struct FilterChain {
    filters: Vec<Box<dyn Filter>>
}

impl FilterChain {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter>) -> &mut Self {
        self.filters.push(filter);
        self
    }
}

impl Filter for FilterChain {
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book> {
        if self.filters.is_empty() {
            return books;
        }

        let mut filtered_books = self.filters[0].do_filter(books);
        if self.filters.len() == 1 {
            return filtered_books;
        }

        for filter in &self.filters[1..] {
            filtered_books = filter.do_filter(filtered_books);
        }

        filtered_books
    }
}

pub struct EmptyIsbnFilter;

impl Filter for EmptyIsbnFilter {
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book> {
        books.into_iter()
            .filter(|book| !book.isbn.is_empty())
            .collect()
    }
}

pub struct OriginDataFilter<R>
where
    R: BookOriginFilterRepository
{
    repository: R
}

impl<R> OriginDataFilter<R>
where
    R: BookOriginFilterRepository
{
    pub fn new(repository: R) -> OriginDataFilter<R> {
        Self { repository }
    }
}

impl<R> Filter for OriginDataFilter<R>
where
    R: BookOriginFilterRepository
{
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book> {
        let filters = self.repository.get_root_filters();
        books.into_iter()
            .filter(|book| filters.iter().all(|filter| filter.borrow().validate(book)))
            .collect()
    }
}
