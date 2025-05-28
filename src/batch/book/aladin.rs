use crate::batch::book::{create_default_filter_chain, ByPublisher, OriginalDataFilter, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{job_builder, Job, JobParameter, Reader};
use crate::item::{Book, BookBuilder, BookRepository, FilterRepository, PublisherRepository, SharedPublisherRepository, Site};
use crate::provider;
use crate::provider::api::{aladin, Client};
use std::rc::Rc;

const PAGE_SIZE: usize = 50;

/// 알라딘 API의 최대 조회 제한
/// 신간 도서가 200건 보다 많아도 200건 까지만 조회 가능하고 그 이후 부터는 1페이지 부터 응답이 반복 된다.
const MAX_RESULT: usize = 200;

pub struct AladinReader {
    client: Rc<aladin::Client>,
    pub_repo: SharedPublisherRepository,
}

impl AladinReader {
    pub fn new(client: Rc<aladin::Client>, pub_repo: SharedPublisherRepository) -> Self {
        Self { client, pub_repo }
    }
}

impl Reader for AladinReader {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        <Self as ByPublisher>::read_books(self, params)
    }
}

impl ByPublisher for AladinReader {
    fn site(&self) -> &Site {
        &Site::Aladin
    }

    fn repository(&self) -> &SharedPublisherRepository {
        &self.pub_repo
    }

    fn by_publisher_keyword(&self, keyword: &str, _: &JobParameter) -> Result<Vec<BookBuilder>, JobReadFailed> {
        let mut result = Vec::new();
        let mut current_fetch_size = 0;
        let mut current_page = 1;
        loop {
            let request = provider::api::Request::builder()
                .page(current_page).size(PAGE_SIZE as i32)
                .query(keyword.to_owned())
                .build().unwrap();

            let response = self.client.get_books(&request).unwrap();
            if !response.books.is_empty() || current_fetch_size < MAX_RESULT {
                current_fetch_size += response.books.len();
                current_page += 1;

                response.books.into_iter().for_each(|b| result.push(b));
            } else {
                break Ok(result);
            }
        }
    }
}

pub fn create_job(
    client: Rc<aladin::Client>,
    publisher_repo: Rc<Box<dyn PublisherRepository>>,
    book_repo: Rc<Box<dyn BookRepository>>,
    filter_repo: Rc<Box<dyn FilterRepository>>,
) -> Job<Book, Book> {
    let filter_chain = create_default_filter_chain()
        .add_filter(Box::new(OriginalDataFilter::new(filter_repo.clone(), Site::Aladin)));

    job_builder()
        .reader(Box::new(AladinReader::new(client.clone(), publisher_repo.clone())))
        .filter(Box::new(filter_chain))
        .writer(Box::new(UpsertBookWriter::new(book_repo.clone())))
        .build()
}