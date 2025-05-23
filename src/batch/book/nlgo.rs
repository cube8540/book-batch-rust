use crate::batch::book::{create_default_filter_chain, retrieve_from_to_in_parameter, ByPublisher, FilterChain, OnlyNewBooksWriter, OriginalDataFilter};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobParameter, PhantomProcessor, Provider, Reader};
use crate::item::{Book, BookBuilder, BookRepository, FilterRepository, PublisherRepository, Site};
use crate::provider;
use crate::provider::api::{nlgo, Client};

const PAGE_SIZE: usize = 500;

pub struct NlgoBookReader<PubRepo: PublisherRepository> {
    client: nlgo::Client,
    publisher_repository: PubRepo,
}

impl<PubRepo: PublisherRepository> NlgoBookReader<PubRepo> {
    pub fn new(client: nlgo::Client, repo: PubRepo) -> Self {
        Self { client, publisher_repository: repo }
    }
}

impl<PubRepo: PublisherRepository> Reader for NlgoBookReader<PubRepo> {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        <Self as ByPublisher<PubRepo>>::read_books(self, params)
    }
}

impl<PubRepo: PublisherRepository> ByPublisher<PubRepo> for NlgoBookReader<PubRepo> {
    fn site(&self) -> &Site {
        &Site::NLGO
    }

    fn repository(&self) -> &PubRepo {
        &self.publisher_repository
    }

    fn by_publisher_keyword(&self, keyword: &str, params: &JobParameter) -> Result<Vec<BookBuilder>, JobReadFailed> {
        let mut result = Vec::new();
        let mut current_page = 1;

        let (from, to) = retrieve_from_to_in_parameter(params)?;
        loop {
            let request = provider::api::Request::builder()
                .page(current_page).size(PAGE_SIZE as i32)
                .query(keyword.to_owned())
                .start_date(from).end_date(to)
                .build().unwrap();

            let response = self.client.get_books(&request).unwrap();
            if !response.books.is_empty() {
                response.books.into_iter().for_each(|b| result.push(b));
                current_page += 1;
            } else {
                break Ok(result);
            }
        }
    }
}

pub fn create_job<PR, BR, FR>(
    client: impl Provider<Item=nlgo::Client>,
    publisher_repo: impl Provider<Item=PR>,
    book_repo: impl Provider<Item=BR>,
    filter_repo: impl Provider<Item=FR>,
) -> Job<Book, Book, NlgoBookReader<PR>, FilterChain, PhantomProcessor<Book>, OnlyNewBooksWriter<BR>>
where
    PR: PublisherRepository + 'static,
    BR: BookRepository + 'static,
    FR: FilterRepository + 'static,
{
    let filter_chain = create_default_filter_chain()
        .add_filter(Box::new(OriginalDataFilter::new(filter_repo.retrieve(), Site::NLGO)));

    Job::builder()
        .reader(NlgoBookReader::new(client.retrieve(), publisher_repo.retrieve()))
        .filter(filter_chain)
        .processor(PhantomProcessor::new())
        .writer(OnlyNewBooksWriter::new(book_repo.retrieve()))
        .build()
        .unwrap()
}
