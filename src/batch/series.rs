mod utils;

use crate::batch::error::JobReadFailed;
use crate::batch::{JobParameter, Processor, Reader};
use crate::item::{Book, Series, SharedBookRepository, SharedSeriesRepository};
use crate::prompt::{NormalizeRequest, NormalizeRequestSaleInfo, SharedPrompt};

const DEFAULT_READ_LIMIT: usize = 50;
const PARAM_NAME_READ_LIMIT: &str = "limit";

/// 시리즈 처리 도중 발생하는 에러 열거
pub enum SeriesProcessError {

    FailedTitleNormalize(String),

    FailedTitleEmbedding(String),

}

/// 시리즈 아이디가 설정 되어 있지 않은 도서를 검색하는 리더
///
/// # Description
/// 시리즈 정보가 할당 되지 않은 도서들을 데이터베이스에서 조회한다.
/// `JobParameter`에서 `limit` 키로 조회할 도서의 수를 지정할 수 있으며 50개를 기본값으로 사용한다.
pub struct UnorganizedBookReader {
    book_repo: SharedBookRepository
}

impl UnorganizedBookReader {
    pub fn new(book_repo: SharedBookRepository) -> Self {
        Self { book_repo }
    }
}

impl Reader for UnorganizedBookReader {
    type Item = Book;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
        let limit = params.get(PARAM_NAME_READ_LIMIT)
            .map(|s| {
                s.parse::<usize>()
                    .map_err(|e| JobReadFailed::InvalidArguments(format!("{}: {} is not a number", PARAM_NAME_READ_LIMIT, e)))
            })
            .unwrap_or_else(|| Ok(DEFAULT_READ_LIMIT))?;

        let books = self.book_repo.find_series_unorganized(limit);
        Ok(books)
    }
}

/// 도서의 시리즈 분류 처리 결과
enum SeriesMappingResult {

    /// 새로운 시리즈를 생성하고 도서와 연결 해야함을 의미한다.
    ///
    /// # Tuple
    /// - `Book`: 시리즈에 연결 되어야 할 도서
    /// - `Series`: 새로 생성될 시리즈 정보
    New(Book, Series),

    /// 기존 시리즈에 도서를 연결 해야함을 의미한다.
    ///
    /// # Tuple
    /// - `Book`: 시리즈에 연결 되어야 할 도서
    /// - `Series`: 연결 대상이 되는 기존 시리즈
    Exists(Book, Series),
}

/// 시리즈 맵핑 프로세서
///
/// # Description
/// LLM 프롬프트를 이용하여 도서의 제목을 정규화하고 데이터베이스에서 가장 유사한 시리즈를 조회해 해당 시리즈로 도서와 연결한다.
/// 만약 유사한 시리즈가 없을 경우 정규화된 제목을 시리즈명으로 사용하여 신규 시리즈를 생성한다.
pub struct SeriesMappingProcessor {
    series_repo: SharedSeriesRepository,
    prompt: SharedPrompt,
}

impl SeriesMappingProcessor {
    pub fn new(series_repo: SharedSeriesRepository, prompt: SharedPrompt) -> Self {
        Self { series_repo, prompt }
    }
}

impl SeriesMappingProcessor {

    /// 도서의 제목을 정규화 하고 새 시리즈를 생성한다.
    ///
    /// # Description
    /// 입력 받은 도서의 제목을 정규화 하여 표준화된 제목을 추출하고 그 제목을 시리즈명으로 가지는 새 시리즈를 하나 생성한다.
    ///
    /// # Parmaeter
    /// - `book`: 제목을 정규화 하고 시리즈화 할 도서 정보
    ///
    /// # Returns
    /// 정규화된 제목으 시리즈명으로 가지는 새 시리즈
    fn normalize(&self, book: &Book) -> Result<Series, SeriesProcessError> {
        let mut request = NormalizeRequest::new(book.title());
        let original = book.originals();

        let mut sale_info_vec = Vec::new();
        for (site, raw) in original {
            let title = utils::extract_title_from_raw(site, raw);
            if let Some(title) = title {
                let mut sale_info = NormalizeRequestSaleInfo::new(&site.to_string(), &title);
                utils::set_sale_info_value(&mut sale_info, site, raw);
                sale_info_vec.push(sale_info);
            }
        }

        if !sale_info_vec.is_empty() {
            request.sale_info = Some(sale_info_vec);
        }

        let normalized = self.prompt.normalize(&request)
            .map_err(|e| SeriesProcessError::FailedTitleNormalize(e.to_string()))?;
        let embedding = self.prompt.embedding(&[normalized.title.as_str()])
            .map_err(|e| SeriesProcessError::FailedTitleEmbedding(e.to_string()))?;
        let embedding = embedding.into_iter().next().unwrap();

        let mut new_series = Series::builder()
            .title(normalized.title.clone())
            .vec(embedding);

        if let Some(set_isbn) = utils::extract_set_isbn_from_book(book) {
            new_series = new_series.isbn(set_isbn);
        }

        Ok(new_series.build().unwrap())
    }
}