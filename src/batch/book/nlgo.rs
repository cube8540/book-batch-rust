use crate::batch::book::{create_default_filter_chain, new_drop_dupl_isbn_filter, new_empty_isbn_filter, new_phantom_processor, retrieve_from_to_in_parameter, ByPublisher, FilterChain, OnlyNewBooksWriter, OriginalDataFilter, PhantomProcessor};
use crate::batch::error::JobReadFailed;
use crate::batch::{Job, JobFactory, JobParameter, Reader};
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

struct NlgoJobFactory<PR, BR, FR>
where
    PR: PublisherRepository + 'static,
    BR: BookRepository + 'static,
    FR: FilterRepository + 'static,
{
    publisher_repository: PR,
    book_repository: BR,
    filter_repository: FR,
    client: nlgo::Client,
}

impl<PR, BR, FR> NlgoJobFactory<PR, BR, FR>
where
    PR: PublisherRepository + 'static,
    BR: BookRepository + 'static,
    FR: FilterRepository + 'static,
{
    pub fn new(publisher_repository: PR, book_repository: BR, filter_repository: FR, client: nlgo::Client) -> Self {
        Self { publisher_repository, book_repository, filter_repository, client }
    }
}

impl<PR, BR, FR> JobFactory<Book, Book> for NlgoJobFactory<PR, BR, FR>
where
    PR: PublisherRepository + 'static,
    BR: BookRepository + 'static,
    FR: FilterRepository + 'static,
{
    type Reader = NlgoBookReader<PR>;
    type Filter = FilterChain;
    type Processor = PhantomProcessor;
    type Writer = OnlyNewBooksWriter<BR>;

    fn create(&self) -> Job<Book, Book, Self::Reader, Self::Filter, Self::Processor, Self::Writer> {
        let filter_chain = create_default_filter_chain()
            .add_filter(Box::new(OriginalDataFilter::new(self.filter_repository.clone(), Site::NLGO)));

        Job::builder()
            .reader(NlgoBookReader::new(self.client.clone(), self.publisher_repository.clone()))
            .filter(filter_chain)
            .processor(new_phantom_processor())
            .writer(OnlyNewBooksWriter::new(self.book_repository.clone()))
            .build()
            .unwrap()
    }
}
