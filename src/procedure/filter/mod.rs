use crate::book::{Book, BookOriginFilterRepository, Site};

pub trait Filter {

    fn do_filter<'job, 'input>(
        &self,
        books: &'input [&'job Book]
    ) -> Vec<&'job Book>
    where 'job: 'input;
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
    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where
        'job: 'input
    {
        if self.filters.is_empty() {
            return books.to_vec()
        }

        let mut filtered_books = self.filters[0].do_filter(books);
        if self.filters.len() == 1 {
            return filtered_books;
        }

        for filter in &self.filters[1..] {
            filtered_books = filter.do_filter(&filtered_books);
        }
        
        filtered_books
    }
}

pub struct EmptyIsbnFilter;

impl Filter for EmptyIsbnFilter {
    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where
        'job: 'input
    {
        books.iter()
            .filter(|book| !book.isbn.is_empty())
            .cloned()
            .collect()
    }
}

pub struct OriginDataFilter<R>
where
    R: BookOriginFilterRepository
{
    repository: R,
    site: Site,
}

impl<R> OriginDataFilter<R>
where
    R: BookOriginFilterRepository
{
    pub fn new(repository: R, site: Site) -> OriginDataFilter<R> {
        Self { repository, site }
    }
}

impl<R> Filter for OriginDataFilter<R>
where
    R: BookOriginFilterRepository
{
    fn do_filter<'job, 'input>(&self, books: &'input [&'job Book]) -> Vec<&'job Book>
    where
        'job: 'input
    {
        let filters = self.repository.get_root_filters();

        if let Some(filter) = filters.get(&self.site) {
            books.iter()
                .filter(|book| {
                    book.origin_data.get(&self.site).map_or(false, |o| filter.borrow().validate(o))
                })
                .cloned()
                .collect()
        } else {
            books.to_vec()
        }
    }
}
