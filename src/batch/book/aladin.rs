use crate::batch::book::ByPublisher;
use crate::batch::error::JobReadFailed;
use crate::batch::{JobParameter, Reader};
use crate::item::{Book, BookBuilder, PublisherRepository, Site};
use crate::provider;
use crate::provider::api::{aladin, Client};

const PAGE_SIZE: usize = 50;

/// 알라딘 API의 최대 조회 제한
/// 신간 도서가 200건 보다 많아도 200건 까지만 조회 가능하고 그 이후 부터는 1페이지 부터 응답이 반복 된다.
const MAX_REUSLTS: usize = 200;

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

    fn by_publisher_keyword(&self, keyword: &str, params: &JobParameter) -> Result<Vec<BookBuilder>, JobReadFailed> {
        let mut result = Vec::new();
        let mut current_fetch_size = 0;
        let mut current_page = 1;
        loop {
            let request = provider::api::Request::builder()
                .page(current_page).size(PAGE_SIZE as i32)
                .query(keyword.to_owned())
                .build().unwrap();

            let response = self.client.get_books(&request).unwrap();
            if !response.books.is_empty() || current_fetch_size < MAX_REUSLTS {
                current_fetch_size += response.books.len();
                current_page += 1;

                response.books.into_iter().for_each(|b| result.push(b));
            } else {
                break Ok(result);
            }
        }
    }
}