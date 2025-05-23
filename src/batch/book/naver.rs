use crate::batch::book::{create_default_filter_chain, new_phantom_processor, retrieve_from_to_in_parameter, FilterChain, OriginalDataFilter, PhantomProcessor, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobFactory, JobParameter, Reader};
use crate::item::{Book, BookRepository, FilterRepository, Site};
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

pub fn create_job<BR, FR>(
    client: naver::Client,
    book_repository: BR,
    filter_repository: FR
) -> Job<Book, Book, NaverReader<BR>, FilterChain, PhantomProcessor, UpsertBookWriter<BR>>
where
    BR: BookRepository,
    FR: FilterRepository,
{
    let filter_chain = create_default_filter_chain()
        .add_filter(Box::new(OriginalDataFilter::new(filter_repository, Site::Naver)));

    Job::builder()
        .reader(NaverReader::new(client, book_repository))
        .filter(filter_chain)
        .processor(new_phantom_processor())
        .writer(UpsertBookWriter::new(book_repository))
        .build()
        .unwrap()
}