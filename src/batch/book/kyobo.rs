use crate::batch::book::{retrieve_from_to_in_parameter, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobParameter, PhantomFilter, PhantomProcessor, Provider, Reader};
use crate::item::{Book, BookRepository};
use crate::provider::html::{kyobo, Client, ParsingError};
use tracing::error;

pub struct KyoboReader<LP, BookRepo>
where
    LP: kyobo::LoginProvider,
    BookRepo: BookRepository
{
    client: kyobo::Client<LP>,
    book_repository: BookRepo,
}

impl<LP, BookRepo> KyoboReader<LP, BookRepo>
where
    LP: kyobo::LoginProvider,
    BookRepo: BookRepository
{
    pub fn new(client: kyobo::Client<LP>, book_repository: BookRepo) -> Self {
        Self { client, book_repository }
    }
}

impl<LP, BookRepo> Reader for KyoboReader<LP, BookRepo>
where
    LP: kyobo::LoginProvider,
    BookRepo: BookRepository
{
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        let (from, to) = retrieve_from_to_in_parameter(params)?;

        let mut result = Vec::new();
        for book in self.book_repository.find_by_pub_between(&from, &to) {
            let response = self.client.get(book.isbn());
            if response.is_ok() {
                let builder = response.unwrap();
                result.push(builder.build().unwrap());
            } else {
                let err = response.unwrap_err();
                match err {
                    // Item을 찾을 수 없다는 에러는 무시 한다.
                    ParsingError::ItemNotFound => {
                        error!("Item not found: {}({})", book.id(), book.isbn());
                    }
                    _ => {
                        return Err(JobReadFailed::UnknownError(err.to_string()));
                    }
                }
            }
        }
        Ok(result)
    }
}

pub fn create_job<LP, BR>(
    client: impl Provider<Item=kyobo::Client<LP>>,
    book_repo: impl Provider<Item=BR>,
) -> Job<Book, Book, KyoboReader<LP, BR>, PhantomFilter<Book>, PhantomProcessor<Book>, UpsertBookWriter<BR>>
where
    LP: kyobo::LoginProvider,
    BR: BookRepository + 'static {
    Job::builder()
        .reader(KyoboReader::new(client.retrieve(), book_repo.retrieve()))
        .filter(PhantomFilter::new())
        .processor(PhantomProcessor::new())
        .writer(UpsertBookWriter::new(book_repo.retrieve()))
        .build()
        .unwrap()
}