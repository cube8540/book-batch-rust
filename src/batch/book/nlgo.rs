use crate::batch::book::{create_default_filter_chain, retrieve_from_to_in_parameter, ByPublisher, OnlyNewBooksWriter, OriginalDataFilter};
use crate::batch::error::JobReadFailed;
use crate::batch::{job_builder, Job, JobParameter, Reader};
use crate::item::{Book, BookBuilder, SharedBookRepository, SharedFilterRepository, SharedPublisherRepository, Site};
use crate::provider;
use crate::provider::api::{nlgo, Client};
use std::rc::Rc;

const PAGE_SIZE: usize = 500;

pub struct NlgoBookReader {
    client: Rc<nlgo::Client>,
    pub_repo: SharedPublisherRepository,
}

impl NlgoBookReader {
    pub fn new(client: Rc<nlgo::Client>, pub_repo: SharedPublisherRepository) -> Self {
        Self { client, pub_repo }
    }
}

impl Reader for NlgoBookReader {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        <Self as ByPublisher>::read_books(self, params)
    }
}

impl ByPublisher for NlgoBookReader {

    fn site(&self) -> &Site {
        &Site::NLGO
    }

    fn repository(&self) -> &SharedPublisherRepository {
        &self.pub_repo
    }

    fn by_publisher_keyword(&self, keyword: &str, params: &JobParameter) -> Result<Vec<BookBuilder>, JobReadFailed> {
        let mut result = Vec::new();
        let mut current_page = 1;

        let (from, to) = retrieve_from_to_in_parameter(params)?;
        loop {
            let request = provider::api::Request::builder()
                .page(current_page).size(PAGE_SIZE as i32)
                .query(keyword.to_owned())
                .start_date(from).end_date(to)
                .build().unwrap();

            let response = self.client.get_books(&request).unwrap();
            if !response.books.is_empty() {
                response.books.into_iter().for_each(|b| result.push(b));
                current_page += 1;
            } else {
                break Ok(result);
            }
        }
    }
}

pub fn create_job(
    client: Rc<nlgo::Client>,
    pub_repo: SharedPublisherRepository,
    book_repo: SharedBookRepository,
    filter_repo: SharedFilterRepository,
) -> Job<Book, Book> {
    let filter_chain = create_default_filter_chain()
        .add_filter(Box::new(OriginalDataFilter::new(filter_repo.clone(), Site::NLGO)));
    
    job_builder()
        .reader(Box::new(NlgoBookReader::new(client.clone(), pub_repo.clone())))
        .filter(Box::new(filter_chain))
        .writer(Box::new(OnlyNewBooksWriter::new(book_repo.clone())))
        .build()
}
