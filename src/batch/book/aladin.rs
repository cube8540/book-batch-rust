use crate::batch::book::{create_default_filter_chain, ByPublisher, FilterChain, OriginalDataFilter, UpsertBookWriter};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobParameter, PhantomProcessor, Provider, Reader};
use crate::item::{Book, BookBuilder, BookRepository, FilterRepository, PublisherRepository, Site};
use crate::provider;
use crate::provider::api::{aladin, Client};

const PAGE_SIZE: usize = 50;

/// 알라딘 API의 최대 조회 제한
/// 신간 도서가 200건 보다 많아도 200건 까지만 조회 가능하고 그 이후 부터는 1페이지 부터 응답이 반복 된다.
const MAX_RESULT: usize = 200;

pub struct AladinReader<PubRepo: PublisherRepository> {
    client: aladin::Client,
    publisher_repository: PubRepo,
}

impl<PubRepo: PublisherRepository> AladinReader<PubRepo> {
    pub fn new(client: aladin::Client, publisher_repository: PubRepo) -> Self {
        Self { client, publisher_repository }
    }
}

impl<PubRepo: PublisherRepository> Reader for AladinReader<PubRepo> {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        <Self as ByPublisher<PubRepo>>::read_books(self, params)
    }
}

impl<PubRepo: PublisherRepository> ByPublisher<PubRepo> for AladinReader<PubRepo> {
    fn site(&self) -> &Site {
        &Site::Aladin
    }

    fn repository(&self) -> &PubRepo {
        &self.publisher_repository
    }

    fn by_publisher_keyword(&self, keyword: &str, _: &JobParameter) -> Result<Vec<BookBuilder>, JobReadFailed> {
        let mut result = Vec::new();
        let mut current_fetch_size = 0;
        let mut current_page = 1;
        loop {
            let request = provider::api::Request::builder()
                .page(current_page).size(PAGE_SIZE as i32)
                .query(keyword.to_owned())
                .build().unwrap();

            let response = self.client.get_books(&request).unwrap();
            if !response.books.is_empty() || current_fetch_size < MAX_RESULT {
                current_fetch_size += response.books.len();
                current_page += 1;

                response.books.into_iter().for_each(|b| result.push(b));
            } else {
                break Ok(result);
            }
        }
    }
}

pub fn create_job<PR, BR, FR>(
    client: impl Provider<Item=aladin::Client>,
    publisher_repo: impl Provider<Item=PR>,
    book_repo: impl Provider<Item=BR>,
    filter_repo: impl Provider<Item=FR>,
) -> Job<Book, Book, AladinReader<PR>, FilterChain, PhantomProcessor<Book>, UpsertBookWriter<BR>>
where
    PR: PublisherRepository + 'static,
    BR: BookRepository + 'static,
    FR: FilterRepository + 'static
{
    let filter_chain = create_default_filter_chain()
        .add_filter(Box::new(OriginalDataFilter::new(filter_repo.retrieve(), Site::Aladin)));

    Job::builder()
        .reader(AladinReader::new(client.retrieve(), publisher_repo.retrieve()))
        .filter(filter_chain)
        .processor(PhantomProcessor::new())
        .writer(UpsertBookWriter::new(book_repo.retrieve()))
        .build()
        .unwrap()
}