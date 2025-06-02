pub mod nlgo;
pub mod naver;
pub mod aladin;
pub mod kyobo;

use crate::batch::error::{JobReadFailed, JobWriteFailed};
use crate::batch::{Filter, FilterChain, JobParameter, Reader, Writer};
use crate::item::{Book, BookBuilder, Publisher, SharedBookRepository, SharedFilterRepository, SharedPublisherRepository, Site};
use crate::{PARAM_NAME_FROM, PARAM_NAME_PUBLISHER_ID, PARAM_NAME_TO};
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};

/// [`JobParameter`]에서 `시작일`과 `종료일`을 얻어 [`NaiveDate`]로 반환한다.
/// 시작일의 키는 `from_dt` 종료일의 키는 `to_dt`를 사용한다. 시작일과 종료일은 `%Y-%m-%d` 포멧으로 파싱하며
/// 파싱 실패시 `JobReadFailed` 에러를 반환한다.
///
/// # Example
/// ```
/// use chrono::NaiveDate;
/// use book_batch_rust::batch::book::retrieve_from_to_in_parameter;
/// use book_batch_rust::batch::JobParameter;
///
/// let mut params = JobParameter::new();
/// params.insert("from".to_owned(), "2025-05-01".to_owned());
/// params.insert("to".to_owned(), "2025-05-31".to_owned());
///
/// let (from, to) = retrieve_from_to_in_parameter(&params).unwrap();
/// assert_eq!(from, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
/// assert_eq!(to, NaiveDate::from_ymd_opt(2025, 5, 31).unwrap());
/// ```
pub fn retrieve_from_to_in_parameter(params: &JobParameter) -> Result<(NaiveDate, NaiveDate), JobReadFailed> {
    let from_str = params.get(PARAM_NAME_FROM);
    let to_str = params.get(PARAM_NAME_TO);

    if from_str.is_none() || to_str.is_none() {
        return Err(JobReadFailed::InvalidArguments("from/to is required".to_owned()));
    }

    let (from_str, to_str) = (from_str.unwrap(), to_str.unwrap());
    let from = NaiveDate::parse_from_str(&from_str, "%Y-%m-%d")
        .map_err(|e| JobReadFailed::InvalidArguments(format!("Invalid from date: {}", e)))?;
    let to = NaiveDate::parse_from_str(&to_str, "%Y-%m-%d")
        .map_err(|e| JobReadFailed::InvalidArguments(format!("Invalid from date: {}", e)))?;

    Ok((from, to))
}

/// [`JobParameter`]에서 `publisher`를 키로 사용하여 출판사 아이디를 얻어온다.
/// 만약 `JobParameter`에 출판사 아이디가 없을 경우 빈 `Vec`를 반환한다.
///
/// 출판사 아이디는 모두 `u64`로 되어 있으며 콤마(,)로 구분 한다. 만약 `u64`로 파싱 할 수 없을 경우 `JobReadFailed` 에러를 반환한다.
///
/// # Example
/// ```
/// use book_batch_rust::batch::book::retrieve_publisher_id_in_parameter;
/// use book_batch_rust::batch::JobParameter;
///
/// let mut parameter = JobParameter::new();
///
/// // 출판사가 아이디가 없을 경우 빈 `Vec`를 반환한다.
/// let publisher_id = retrieve_publisher_id_in_parameter(&parameter).unwrap();
/// assert_eq!(publisher_id, Vec::<u64>::new());
///
/// // 출판사 아이디를 콤마(,)로 구분하고 `u64` 타입으로 변환하여 반환한다.
/// parameter.insert("publisher".to_owned(), "1, 2, 3".to_owned());
/// let publisher_id = retrieve_publisher_id_in_parameter(&parameter).unwrap();
/// assert_eq!(publisher_id, vec![1,2,3]);
/// ```
pub fn retrieve_publisher_id_in_parameter(params: &JobParameter) -> Result<Vec<u64>, JobReadFailed> {
    let publisher_id = params.get(PARAM_NAME_PUBLISHER_ID);

    if publisher_id.is_none() {
        return Ok(Vec::new());
    }

    let publisher_id_str = publisher_id.unwrap().split(',');
    let publisher_ids: Result<Vec<u64>, JobReadFailed> = publisher_id_str
        .map(|s| {
            s.trim().parse::<u64>()
                .map_err(|e| JobReadFailed::InvalidArguments(e.to_string()))
        })
        .collect();

    match publisher_ids {
        Ok(ids) => Ok(ids),
        Err(err) => Err(err),
    }
}

pub trait ByPublisher: Reader<Item=Book> {

    fn site(&self) -> &Site;

    fn repository(&self) -> &SharedPublisherRepository;

    fn by_publisher_keyword(&self, keyword: &str, params: &JobParameter) -> Result<Vec<BookBuilder>, JobReadFailed>;

    fn load_publisher(&self, params: &JobParameter) -> Result<Vec<Publisher>, JobReadFailed> {
        let publisher_id = retrieve_publisher_id_in_parameter(params)?;
        let publisher = if !publisher_id.is_empty() {
            self.repository().find_by_id(&publisher_id)
        } else {
            self.repository().get_all()
        };
        Ok(publisher)
    }

    fn read_books(&self, params: &JobParameter) -> Result<Vec<Book>, JobReadFailed> {
        let publishers = self.load_publisher(params)?;
        let mut results = Vec::new();

        for publisher in publishers {
            let keywords = publisher.keywords()
                .get(self.site())
                .ok_or_else(|| JobReadFailed::InvalidArguments(format!("No keywords for site {:?}", self.site())))?;
            for keyword in keywords {
                let books = self.by_publisher_keyword(keyword, params)?;
                let books: Vec<Book> = books.into_iter()
                    .map(|book| book.publisher_id(publisher.id()).build().unwrap())
                    .collect();

                results.extend(books);
            }
        }
        Ok(results)
    }
}

pub struct EmptyIsbnFilter;

pub fn new_empty_isbn_filter() -> EmptyIsbnFilter {
    EmptyIsbnFilter {}
}

impl Filter for EmptyIsbnFilter {
    type Item = Book;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
        items.into_iter()
            .filter(|item| !item.isbn().is_empty())
            .collect()
    }
}

pub struct DropDuplicateIsbnFilter;

pub fn new_drop_duplicate_isbn_filter() -> DropDuplicateIsbnFilter {
    DropDuplicateIsbnFilter {}
}

impl Filter for DropDuplicateIsbnFilter {
    type Item = Book;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
        let mut isbn_set: HashSet<String> = HashSet::new();
        let mut filtered_books: Vec<Self::Item> = Vec::new();

        for book in items {
            if !isbn_set.contains(book.isbn()) {
                isbn_set.insert(book.isbn().to_owned());
                filtered_books.push(book);
            }
        }

        filtered_books
    }
}

pub struct OriginalDataFilter {
    repository: SharedFilterRepository,
    site: Site
}

impl OriginalDataFilter {
    pub fn new(repository: SharedFilterRepository, site: Site) -> OriginalDataFilter {
        OriginalDataFilter {
            repository,
            site
        }
    }
}

impl Filter for OriginalDataFilter {
    type Item = Book;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
        let mut filters = self.repository.find_by_site(&self.site).into_iter()
            .map(|rule| rule.to_predicate());

        items.into_iter()
            .filter(|book| {
                book.originals().get(&self.site)
                    .map(|o| filters.all(|f| f.test(o)))
                    .unwrap_or(true)
            })
            .collect()
    }
}

pub fn create_default_filter_chain() -> FilterChain<Book> {
    FilterChain::new()
        .add_filter(Box::new(new_empty_isbn_filter()))
        .add_filter(Box::new(new_drop_duplicate_isbn_filter()))
}

pub struct OnlyNewBooksWriter {
    repo: SharedBookRepository,
}

impl OnlyNewBooksWriter {
    pub fn new(repo: SharedBookRepository) -> Self {
        Self {
            repo,
        }
    }
}

impl Writer for OnlyNewBooksWriter {
    type Item = Book;

    fn do_write(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>> {
        let exists_in_db = retrieve_exists_book_in_db(&self.repo, &items);

        let new_books = items.into_iter()
            .filter(|b| !exists_in_db.contains_key(b.isbn()))
            .collect::<Vec<_>>();

        let wrote = self.repo.save_books(&new_books);
        if wrote.len() > 0 {
            Ok(())
        } else {
            Err(JobWriteFailed::new(new_books, "No new books to write"))
        }
    }
}

pub struct UpsertBookWriter {
    repo: SharedBookRepository,
}

impl UpsertBookWriter {
    pub fn new(repo: SharedBookRepository) -> Self {
        Self {
            repo,
        }
    }
}

impl Writer for UpsertBookWriter {
    type Item = Book;

    fn do_write(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>> {
        let exists_in_db = retrieve_exists_book_in_db(&self.repo, &items);

        let mut new_books = Vec::new();
        for book in items {
            if !exists_in_db.contains_key(book.isbn()) {
                new_books.push(book);
            } else {
                let db_book = exists_in_db.get(book.isbn()).unwrap();
                let merged_book = db_book.merge(&book);
                let updated_count = self.repo.update_book(&merged_book);
                if updated_count <= 0 {
                    return Err(JobWriteFailed::new(vec![merged_book], "Failed to update book"));
                }
            }
        }

        let wrote = self.repo.save_books(&new_books);
        if wrote.len() > 0 {
            Ok(())
        } else {
            Err(JobWriteFailed::new(new_books, "No new books to write"))
        }
    }
}

fn retrieve_exists_book_in_db(repo: &SharedBookRepository, books: &[Book]) -> HashMap<String, Book> {
    let books_isbn = books.iter().map(|b| b.as_ref().isbn()).collect::<Vec<_>>();
    repo.find_by_isbn(&books_isbn).into_iter()
        .map(|b| (b.isbn().to_owned(), b))
        .collect::<HashMap<_, _>>()
}