use std::collections::HashSet;
use crate::item::{Book, FilterRepository, Site};

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
            .filter(|book| !book.isbn().is_empty())
            .collect()
    }
}

pub struct DropDuplicatedIsbnFilter;

impl Filter for DropDuplicatedIsbnFilter {
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book> {
        let mut isbn_set = HashSet::new();
        let mut filtered_books = Vec::new();

        for book in books {
            if !isbn_set.contains(book.isbn()) {
                isbn_set.insert(book.isbn().to_owned());
                filtered_books.push(book);
            }
        }

        filtered_books
    }
}

pub struct OriginDataFilter<R>
where
    R: FilterRepository
{
    repository: R,
    site: Site
}

impl<R> OriginDataFilter<R>
where
    R: FilterRepository
{
    pub fn new(repository: R, site: Site) -> OriginDataFilter<R> {
        Self { repository, site }
    }
}

impl<R> Filter for OriginDataFilter<R>
where
    R: FilterRepository
{
    fn do_filter(&self, books: Vec<Book>) -> Vec<Book> {
        let mut filters = self.repository.find_by_site(&self.site).into_iter()
            .map(|f| f.to_predicate());
        books.into_iter()
            .filter(|book| { 
                book.originals().get(&self.site)
                    .map(|o| filters.all(|f| f.test(o)))
                    .unwrap_or(true)
            })
            .collect()
    }
}
