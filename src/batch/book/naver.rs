use crate::batch::book::{new_phantom_filter, new_phantom_processor, retrieve_from_to_in_parameter, PhantomFilter, PhantomProcessor, Provider, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobParameter, Reader};
use crate::item::{Book, BookRepository};
use crate::provider;
use crate::provider::api::{naver, Client};

pub struct NaverReader<BookRepo: BookRepository> {
    client: naver::Client,
    book_repository: BookRepo
}

impl<BookRepo: BookRepository> NaverReader<BookRepo> {
    pub fn new(client: naver::Client, book_repository: BookRepo) -> Self {
        Self { client, book_repository }
    }
}

impl<BookRepo: BookRepository> Reader for NaverReader<BookRepo> {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        let (from, to) = retrieve_from_to_in_parameter(params)?;
        let results = self.book_repository.find_by_pub_between(&from, &to).into_iter()
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

pub fn create_job<BR>(
    client: impl Provider<Item=naver::Client>,
    book_repo: impl Provider<Item=BR>,
) -> Job<Book, Book, NaverReader<BR>, PhantomFilter, PhantomProcessor, UpsertBookWriter<BR>>
where
    BR: BookRepository + 'static,
{
    Job::builder()
        .reader(NaverReader::new(client.retrieve(), book_repo.retrieve()))
        .filter(new_phantom_filter())
        .processor(new_phantom_processor())
        .writer(UpsertBookWriter::new(book_repo.retrieve()))
        .build()
        .unwrap()
}