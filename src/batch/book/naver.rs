use crate::batch::book::{retrieve_from_to_in_parameter, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{job_builder, Job, JobParameter, Reader};
use crate::item::{Book, SharedBookRepository};
use crate::provider;
use crate::provider::api::{naver, Client};
use std::rc::Rc;

pub struct NaverReader {
    client: Rc<naver::Client>,
    book_repo: SharedBookRepository
}

impl NaverReader {
    pub fn new(client: Rc<naver::Client>, book_repo: SharedBookRepository) -> Self {
        Self { client, book_repo }
    }
}

impl Reader for NaverReader {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        let (from, to) = retrieve_from_to_in_parameter(params)?;
        let results = self.book_repo.find_by_pub_between(&from, &to).into_iter()
            .flat_map(|book| {
                let request = provider::api::Request::builder()
                    .query(book.isbn().to_owned())
                    .build().unwrap();

                self.client.get_books(&request).unwrap().books
                    .into_iter()
                    .map(|b| b.build().unwrap())
            })
            .collect();
        Ok(results)
    }
}

pub fn create_job(
    client: Rc<naver::Client>,
    book_repo: SharedBookRepository,
) -> Job<Book, Book> {
    job_builder()
        .reader(Box::new(NaverReader::new(client.clone(), book_repo.clone())))
        .writer(Box::new(UpsertBookWriter::new(book_repo.clone())))
        .build()
}