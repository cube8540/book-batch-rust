use crate::batch::book::{retrieve_from_to_in_parameter, UpsertBookWriter};
use crate::batch::error::{JobProcessFailed, JobReadFailed};
use crate::batch::{Job, JobParameter, PhantomFilter, Processor, Reader};
use crate::item::{Book, RawValue, SharedBookRepository, Site};
use crate::provider::html::{kyobo, Client, ParsingError};
use std::rc::Rc;
use tracing::{error, warn};

pub struct KyoboReader<LP>
where
    LP: kyobo::LoginProvider,
{
    client: Rc<kyobo::Client<LP>>,
    book_repo: SharedBookRepository,
}

impl<LP> KyoboReader<LP>
where
    LP: kyobo::LoginProvider,
{
    pub fn new(client: Rc<kyobo::Client<LP>>, book_repo: SharedBookRepository) -> Self {
        Self { client, book_repo }
    }
}

impl<LP> Reader for KyoboReader<LP>
where
    LP: kyobo::LoginProvider,
{
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        let (from, to) = retrieve_from_to_in_parameter(params)?;

        let mut result = Vec::new();
        for book in self.book_repo.find_by_pub_between(&from, &to) {
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
    api: Rc<kyobo::KyoboAPI>
}

impl KyoboAddSeriesOriginal {
    pub fn new(api: Rc<kyobo::KyoboAPI>) -> Self {
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
        if book_items.is_ok() {
            let book_items = book_items.unwrap();
            let series = book_items.iter()
                .map(|book_item| book_item.to_raw_val())
                .collect::<Vec<_>>();

            let new_book = item.to_builder()
                .add_original_raw(Site::KyoboBook, "series", RawValue::Array(series));

            Ok(new_book.build().unwrap())
        } else {
            warn!("Failed to get series list: {}({})", item.id(), item.isbn());
            Ok(item)
        }
    }
}

pub fn create_job<LP>(
    client: Rc<kyobo::Client<LP>>,
    api: Rc<kyobo::KyoboAPI>,
    book_repo: SharedBookRepository,
) -> Job<Book, Book>
where
    LP: kyobo::LoginProvider + 'static,
{
    Job::builder()
        .reader(KyoboReader::new(client.clone(), book_repo.clone()))
        .filter(PhantomFilter::new())
        .processor(KyoboAddSeriesOriginal::new(api.clone()))
        .writer(UpsertBookWriter::new(book_repo.clone()))
        .build()
        .unwrap()
}