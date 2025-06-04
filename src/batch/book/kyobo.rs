use crate::batch::book::{retrieve_from_to_in_parameter, retrieve_isbn_in_parameter, UpsertBookWriter};
use crate::batch::error::{JobProcessFailed, JobReadFailed};
use crate::batch::{job_builder, Job, JobParameter, Processor, Reader};
use crate::item::{Book, RawValue, SharedBookRepository, Site};
use crate::provider::html::{kyobo, Client, ParsingError};
use std::rc::Rc;
use tracing::{error, warn};
use crate::PARAM_NAME_ISBN;

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

impl <LP> Reader for KyoboReader<LP>
where
    LP: kyobo::LoginProvider,
{
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        let mut result = Vec::new();

        let isbn_vec = if params.contains_key(PARAM_NAME_ISBN) {
            retrieve_isbn_in_parameter(params)?
        } else {
            let (from, to) = retrieve_from_to_in_parameter(params)?;
            self.book_repo.find_by_pub_between(&from, &to).iter()
                .map(|book| book.isbn().to_owned())
                .collect()
        };

        for isbn in isbn_vec {
            let response = self.client.get(&isbn)
                .map(|builder| builder.build().unwrap());
            match response {
                Ok(book) => result.push(book),
                Err(err) => {
                    match err {
                        // ItemNotFound (데이터를 찾을 수 없음) 로그를 남기고 작업을 진핸한다.
                        ParsingError::ItemNotFound => error!("Item(isbn) not found: {}", isbn),
                        _ => return Err(JobReadFailed::UnknownError(err.to_string()))
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
    job_builder()
        .reader(Box::new(KyoboReader::new(client.clone(), book_repo.clone())))
        .processor(Box::new(KyoboAddSeriesOriginal::new(api.clone())))
        .writer(Box::new(UpsertBookWriter::new(book_repo.clone())))
        .build()
}