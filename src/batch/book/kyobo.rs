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

pub fn create_job<LP>(
    client: Rc<kyobo::Client<LP>>,
    book_repo: SharedBookRepository,
) -> Job<Book, Book>
where
    LP: kyobo::LoginProvider + 'static,
{
    job_builder()
        .reader(Box::new(KyoboReader::new(client.clone(), book_repo.clone())))
        .writer(Box::new(UpsertBookWriter::new(book_repo.clone())))
        .build()
}