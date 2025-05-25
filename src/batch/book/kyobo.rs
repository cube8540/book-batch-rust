use crate::batch::book::{retrieve_from_to_in_parameter, UpsertBookWriter};
use crate::batch::error::{JobProcessFailed, JobReadFailed};
use crate::batch::{Job, JobParameter, PhantomFilter, Processor, Provider, Reader};
use crate::item::{Book, BookRepository, RawValue, Site};
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

pub struct KyoboAddSeriesOriginal {
    api: kyobo::KyoboAPI
}

impl KyoboAddSeriesOriginal {
    pub fn new(api: kyobo::KyoboAPI) -> Self {
        Self { api }
    }
}

impl Processor for KyoboAddSeriesOriginal {
    type In = Book;
    type Out = Book;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
        let item_id = item.originals().get(&Site::KyoboBook);
        if item_id.is_none() {
            return Ok(item);
        }
        let item_id = item_id.unwrap().get("item_id");
        if item_id.is_none() {
            return Ok(item);
        }

        let item_id = match item_id.unwrap() {
            RawValue::Text(s) => s.as_str(),
            _ => {
                return Err(JobProcessFailed::new(item, "item_id is not text".to_string()))
            }
        };
        let book_items = self.api.get_series_list(item_id);
        if book_items.is_err() {
            return Err(JobProcessFailed::new(item, "failed to get series list".to_string()));
        }
        let book_items = book_items.unwrap();
        let series = book_items.iter()
            .map(|book_item| book_item.to_raw_val())
            .collect::<Vec<_>>();

        let new_book = item.to_builder()
            .add_original_raw(Site::KyoboBook, "series", RawValue::Array(series));

        Ok(new_book.build().unwrap())
    }
}

pub fn create_job<LP, BR>(
    client: impl Provider<Item=kyobo::Client<LP>>,
    api: impl Provider<Item=kyobo::KyoboAPI>,
    book_repo: impl Provider<Item=BR>,
) -> Job<Book, Book, KyoboReader<LP, BR>, PhantomFilter<Book>, KyoboAddSeriesOriginal, UpsertBookWriter<BR>>
where
    LP: kyobo::LoginProvider,
    BR: BookRepository + 'static {
    Job::builder()
        .reader(KyoboReader::new(client.retrieve(), book_repo.retrieve()))
        .filter(PhantomFilter::new())
        .processor(KyoboAddSeriesOriginal::new(api.retrieve()))
        .writer(UpsertBookWriter::new(book_repo.retrieve()))
        .build()
        .unwrap()
}