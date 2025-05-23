use crate::batch::book::{retrieve_from_to_in_parameter, PhantomFilter, PhantomProcessor, Provider, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobParameter, Reader};
use crate::item::{Book, BookRepository};
use crate::provider::html::{kyobo, Client};

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
        self.book_repository.find_by_pub_between(&from, &to).into_iter()
            .filter(|book| book.actual_pub_date().is_some())
            .map(|book| {
                self.client.get(book.isbn())
                    .map(|parsed_book| parsed_book.build().unwrap())
                    .map_err(|e| JobReadFailed::UnknownError(e.to_string()))
            })
            .collect()
    }
}

pub fn create_job<LP, BR>(
    client: impl Provider<Item=kyobo::Client<LP>>,
    book_repo: impl Provider<Item=BR>,
) -> Job<Book, Book, KyoboReader<LP, BR>, PhantomFilter, PhantomProcessor, UpsertBookWriter<BR>>
where
    LP: kyobo::LoginProvider,
    BR: BookRepository + 'static {
    Job::builder()
        .reader(KyoboReader::new(client.retrieve(), book_repo.retrieve()))
        .filter(PhantomFilter)
        .processor(PhantomProcessor)
        .writer(UpsertBookWriter::new(book_repo.retrieve()))
        .build()
        .unwrap()
}