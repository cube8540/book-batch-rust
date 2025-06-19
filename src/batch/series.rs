use crate::batch::error::{JobProcessFailed, JobReadFailed, JobWriteFailed};
use crate::batch::{job_builder, Job, JobParameter, Processor, ProcessorChain, Reader, Writer};
use crate::item::{raw_utils, Book, RawDataKind, Series, SharedBookRepository, SharedSeriesRepository, Site};
use crate::prompt::{NormalizeRequest, NormalizeRequestSaleInfo, SeriesSimilarRequest, SeriesSimilarRequestBookInfo, SharedPrompt};
use crate::provider::api::nlgo;
use crate::PARAM_NAME_LIMIT;
use std::fmt::{Display, Formatter};

const DEFAULT_READ_LIMIT: usize = 50;

/// 기준 유사도 기본값
const DEFAULT_SIMILARITY_SCORE: f64 = 0.90;

/// 시리즈 소속 여부 재검토 기준 유사도 기본값
const DEFAULT_SERIES_SIMILARITY_SCORE: f64 = 0.45;

/// 시리즈 처리 도중 발생하는 에러 열거
#[derive(Debug)]
pub enum SeriesProcessError {

    FailedTitleNormalize(String),

    FailedTitleEmbedding(String),

}

impl Display for SeriesProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SeriesProcessError::FailedTitleNormalize(msg) => write!(f, "failed title normalize {}", msg),
            SeriesProcessError::FailedTitleEmbedding(msg) => write!(f, "failed title embedding {}", msg),
        }
    }
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
        let limit = params.get(PARAM_NAME_LIMIT)
            .map(|s| {
                s.parse::<usize>()
                    .map_err(|e| JobReadFailed::InvalidArguments(format!("{}: {} is not a number", PARAM_NAME_LIMIT, e)))
            })
            .unwrap_or_else(|| Ok(DEFAULT_READ_LIMIT))?;

        let books = self.book_repo.find_series_unorganized(limit);
        Ok(books)
    }
}

/// 가장 유사한 시리즈와 유사도를 저장하는 구조체
#[derive(Debug)]
pub struct MostSimilarSeries {

    /// 가장 유사했던 시리즈
    pub series: Series,

    /// 유사도 점수
    pub score: f64,
}

/// 도서의 시리즈 분류 처리 결과
#[derive(Debug)]
pub enum SeriesMappingResult {

    /// 새로운 시리즈를 생성하고 도서와 연결 해야함을 의미한다.
    ///
    /// # Tuple
    /// - `0`: 시리즈에 연결 되어야 할 도서
    /// - `1`: 새로 생성될 시리즈 정보
    /// - `2`: 가장 유사했던 시리즈와 그 유사도
    New(Book, Series, Option<MostSimilarSeries>),

    /// 기존 시리즈에 도서를 연결 해야함을 의미한다.
    ///
    /// # Tuple
    /// - `0`: 시리즈에 연결 되어야 할 도서
    /// - `1`: 연결 대상이 되는 기존 시리즈
    Exists(Book, Series),
}

/// 시리즈 검색 객체
///
/// # Description
/// 시리즈 정규화를 위해 데이터베이스에 저장된 기존 시리즈를 검색하는 퍼사드 객체
struct SeriesFinder {
    series_repo: SharedSeriesRepository,
}

impl SeriesFinder {

    /// 시리즈 ISBN을 입력 받아 데이터베이스에 저장된 기존 시리즈를 검색한다.
    ///
    /// # Parameters
    /// - isbn: 시리즈 ISBN
    fn by_isbn(&self, isbn: &str) -> Option<Series> {
        let series_vec = self.series_repo.find_by_isbn(&[isbn]);
        series_vec.into_iter().next()
    }

    /// 입력 받은 시리즈와 제목이 가장 유사한 시리즈를 데이터베이스에서 하나 찾는다.
    ///
    /// # Flow
    /// 1. 코사인 유사도를 기준으로 가장 유사한 시리즈 2개를 검색한다.
    /// 2. 아래의 조건으로 반환값을 결정 한다:
    ///     - 입력 시리즈에 ISBN이 있는 경우:
    ///       * 검색된 시리즈 중 입력 시리즈의 ISBN과 다른 ISBN을 가지는 시리즈 반환
    ///     - 입력 시리즈에 ISBN이 없는 경우:
    ///       * 항상 첫 번째(0번) 시리즈 반환
    ///
    /// ## 특수한 반환값 결정 조건이 필요한 이유
    /// 하나의 도서가 여러 컨텐츠(예: 소설, 만화 등)로 출간될 때 각 컨텐츠별로 서로 다른 ISBN이 부여 될 수 있으며,
    /// 제목은 동일하거나 매우 유사할 수 있다. 따라서 단순히 제목의 유사도만으로 비교하면 실제로는 다른 형태의 시리즈를 동일한
    /// 시리즈로 잘못 판단 할 수 있어 이러한 오류를 방지하기 위해 ISBN 존재 여부를 추가로 확인하는 조건이 필요하다.
    ///
    /// # Note
    /// 이 함수는 검색된 시리즈와 입력 시리즈의 ISBN이 중복되지 않기 위해 [`self.by_isbn`]으로 시리즈를 찾지 못했거나
    /// ISBN이 없는 시리즈를 처리할 때만 사용해야 한다.
    ///
    /// # Parameters
    /// - series: 데이터베이스에 찾고 싶은 시리즈 정보
    fn similarity(&self, series: &Series) -> Option<(Series, Option<f64>)> {
        let series_vec = self.series_repo.similarity(series, 2);
        if series_vec.is_empty() {
            return None;
        }

        let mut series_vec = series_vec.into_iter();
        if let Some(input_series_isbn) = series.isbn().clone() {
            series_vec
                .find(|(s, _)| s.isbn().is_none() || s.isbn().clone().unwrap() != input_series_isbn)
        } else {
            series_vec.next()
        }
    }
}

/// 시리즈 맵핑 프로세서
///
/// # Description
/// LLM 프롬프트를 이용하여 도서의 제목을 정규화하고 데이터베이스에서 가장 유사한 시리즈를 조회해 해당 시리즈로 도서와 연결한다.
/// 만약 유사한 시리즈가 없을 경우 정규화된 제목을 시리즈명으로 사용하여 신규 시리즈를 생성한다.
pub struct SeriesMappingProcessor {
    series_finder: SeriesFinder,
    prompt: SharedPrompt,

    /// 기준 유사도
    ///
    /// # Description
    /// 시리즈를 연결 할 때 사용할 기준 유사도로 여기에 설정된 값 이상의 유사도를 가질 경우 같은 시리즈로 판단하고 도서를 연결한다.
    /// 0 ~ 1 사이의 값을 입력하며 값이 높을수록 더욱 유사한 것을 나타낸다.
    pub similar_score: f64,
}

impl SeriesMappingProcessor {
    pub fn new(series_repo: SharedSeriesRepository, prompt: SharedPrompt) -> Self {
        Self {
            series_finder: SeriesFinder { series_repo },
            prompt,
            similar_score: DEFAULT_SIMILARITY_SCORE
        }
    }
}

impl SeriesMappingProcessor {

    /// 도서의 제목을 정규화 하고 새 시리즈를 생성한다.
    ///
    /// # Description
    /// 입력 받은 도서의 제목을 정규화 하여 표준화된 제목을 추출하고 임베딩 하여 그 제목을 시리즈명으로 가지는 새 시리즈를 하나 생성한다.
    ///
    /// # Parmaeter
    /// - `book`: 제목을 정규화 하고 시리즈화 할 도서 정보
    ///
    /// # Returns
    /// 정규화된 제목을 시리즈명으로 가지는 새 시리즈
    fn normalize(&self, book: &Book) -> Result<Series, SeriesProcessError> {
        let request = convert_book_to_normalize_request(book);

        let normalized = self.prompt.normalize(&request)
            .map_err(|e| SeriesProcessError::FailedTitleNormalize(e.to_string()))?;

        let embedding = self.prompt.embedding(&[normalized.title.clone()])
            .map_err(|e| SeriesProcessError::FailedTitleEmbedding(e.to_string()))?;
        let embedding = embedding.into_iter().next().unwrap();

        let mut new_series = Series::builder()
            .title(normalized.title.clone())
            .vec(embedding);

        if let Some(set_isbn) = retrieve_nlgo_set_isbn(book) {
            new_series = new_series.isbn(set_isbn);
        }

        Ok(new_series.build().unwrap())
    }
}

impl Processor for SeriesMappingProcessor {
    type In = Book;
    type Out = SeriesMappingResult;

    /// 도서가 속할 시리즈를 찾고 맵핑 결과로 변환한다.
    ///
    /// # Description
    /// 전달 받은 도서명을 LLM을 통해 정규화 하고 데이터베이스에서 도서가 속할 시리즈를 찾아 그 맵핑 결과를 반환한다.
    ///
    /// # Flow
    /// 1. 도서에 시리즈의 ISBN이 있을 경우 데이터베이스에서 검색한다.
    /// 데이터베이스에 시리즈가 있을 경우 그 시리즈에 맵핑하라는 결과를 반환한다.
    /// 2. 도서명을 정규화하고 임베딩 하여 데이터베이스에서 가장 유사한 시리즈를 하나 검색 한다.
    /// 3. 검색된 시리즈의 유사도가 설정된 기준 유사도를 넘을 경우 해당 시리즈로 맵핑하라는 결과를 반환하며,
    /// 넘지 못할 경우 새 시리즈를 생성하라는 결과를 반환한다.
    ///
    /// # Note
    /// - 시리즈 ISBN은 도서의 원본 데이터에서 가져오며, `국립중앙도서관(NLGO)`의 `set_isbn`을 사용한다.
    /// - 유사도 검색시 사용되는 알고리즘은 코사인 유사도로 0에 가까울수록 유사함을 나타낸다.
    /// 점수 환산시에는 1에서 유사도를 뺀 값을 점수로 한다.
    ///
    /// # Return
    /// - [`SeriesMappingResult::New`]: 설정된 유사도 이상의 유사한 시리즈를 찾지 못하였을 경우
    /// - [`SeriesMappingResult::Exists`]: 시리즈 ISBN을 데이터베이스에서 찾았거나
    /// 설정된 유사도 이상의 시리즈를 찾았을 경우
    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
        if let Some(set_isbn) = retrieve_nlgo_set_isbn(&item) {
            let series = self.series_finder.by_isbn(&set_isbn);
            if let Some(series) = series {
                return Ok(SeriesMappingResult::Exists(item, series));
            }
        }

        let normalized = self.normalize(&item);
        if normalized.is_err() {
            return Err(JobProcessFailed::new(item, normalized.unwrap_err().to_string()));
        }
        let new_series = normalized.unwrap();

        let most_similar_series = self.series_finder
            .similarity(&new_series)
            .filter(|(_, similar)| similar.is_some())
            .map(|(series, similar)| (series, 1.0 - similar.unwrap()));

        match most_similar_series {
            Some((exists_series, score)) => {
                if score >= self.similar_score {
                    Ok(SeriesMappingResult::Exists(item, exists_series))
                } else {
                    Ok(SeriesMappingResult::New(item, new_series, Some(MostSimilarSeries { series: exists_series, score })))
                }
            }
            None => Ok(SeriesMappingResult::New(item, new_series, None))
        }
    }
}

/// 시리즈 소속 여부 검증 프로세서
///
/// # Description
/// 신간 도서가 기존 시리즈에 속하는지 LLM을 활용하여 재검증 하는 프로세서
/// 단순 제목 비교로는 시리즈 판단이 어려운 경우를 위한 최종 검증 단계로 활용한다.
///
/// # How to work
/// 1. 이전 단계에서 새 시리즈로 분류된 도서([`SeriesMappingResult::New`])를 대상으로 한다.
/// 2. 해당 도서와 가장 유사했던 기존 시리즈의 도서 목록을 함께 LLM에 전달한다.
/// 3. LLM이 신간 도서의 시리즈 소속 여부를 최종 판단한다.
///
/// # Why
/// 동일한 도서라도 판매처마다 제목을 다르게 등록할 수 있어 정규화 후에도 데이터베이스에 기록된 시리즈명과 차이가 있을 수 있어
/// 유사도 검사만으로는 한계가 있다. 이 때 LLM을 이용하여 도서 목록 전체를 검토해 시리즈 소속 여부를 비교적 정확하게 판단한다.
pub struct BelongToSeriesProcessor {
    book_repo: SharedBookRepository,
    prompt: SharedPrompt,

    /// 기준 유사도
    ///
    /// # Description
    /// 시리즈 소속 여부를 재검토할지 여부를 확인하는 기준 유사도로 이전 단계에서 새 시리즈 생성이 필요한 도서로 분류 되었지만 가장 높은 유사도를
    /// 가진 시리즈의 유사도가 이 값을 넘었을 때 해당 시리즈의 도서 목록과 함께 LLM에 입력으로 사용하여 시리즈에 속하는지 재검토 한다.
    ///
    /// # Note
    /// 0 ~ 1 사이의 값을 사용한다.
    pub similar_score: f64,
}

impl BelongToSeriesProcessor {
    pub fn new(book_repo: SharedBookRepository, prompt: SharedPrompt) -> Self {
        Self { book_repo, prompt, similar_score: DEFAULT_SERIES_SIMILARITY_SCORE }
    }
}

impl Processor for BelongToSeriesProcessor {
    type In = SeriesMappingResult;
    type Out = SeriesMappingResult;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
        match item {
            SeriesMappingResult::New(book, new, most_similar) => {
                if most_similar.is_none() {
                    return Ok(SeriesMappingResult::New(book, new, None));
                }
                let most_similar = most_similar.unwrap();
                if most_similar.score < self.similar_score {
                    return Ok(SeriesMappingResult::New(book, new, Some(most_similar)));
                }

                let most_similar_series_books = self.book_repo.find_by_series_id(most_similar.series.id());
                let series_books = most_similar_series_books.iter()
                    .map(convert_series_similar_request_book_info)
                    .collect();
                let new_book = convert_series_similar_request_book_info(&book);

                let request = SeriesSimilarRequest { new: new_book, series: series_books, };
                let response = self.prompt.series_similar(&request);

                if response.is_err() {
                    let err = response.unwrap_err();
                    return Err(JobProcessFailed::new(SeriesMappingResult::New(book, new, Some(most_similar)), err.to_string()));
                }

                if response.unwrap() {
                    Ok(SeriesMappingResult::Exists(book, most_similar.series))
                } else {
                    Ok(SeriesMappingResult::New(book, new, Some(most_similar)))
                }
            }
            _ => Ok(item)
        }
    }
}

/// 시리즈를 저장하는 객체
///
/// # Description
/// 시리즈 맵핑 결과를 받아 신규 시리즈를 저장하거나, 도서의 시리즈 아이디를 연결된 시리즈의 아이디로 업데이트 한다.
pub struct SeriesWriter {
    series_repo: SharedSeriesRepository,
    book_repo: SharedBookRepository,
}

impl SeriesWriter {
    pub fn new(series_repo: SharedSeriesRepository, book_repo: SharedBookRepository) -> Self {
        Self { series_repo, book_repo }
    }
}

impl Writer for SeriesWriter {
    type Item = SeriesMappingResult;

    fn do_write(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>> {
        for item in items.into_iter() {
            match item {
                SeriesMappingResult::Exists(mut book, exists_series) => {
                    book.set_series_id(exists_series.id());
                    self.book_repo.update_book(&book);
                }
                SeriesMappingResult::New(mut book, new_series, _) => {
                    let insert_series = vec![new_series];
                    let inserted_series = self.series_repo
                        .new_series(&insert_series).into_iter().next();

                    if inserted_series.is_none() {
                        let series = insert_series.into_iter().next().unwrap();
                        let err_val = vec![SeriesMappingResult::New(book, series, None)];
                        return Err(JobWriteFailed::new(err_val, "시리즈가 저장 되지 않았습니다."))
                    }

                    book.set_series_id(inserted_series.unwrap().id());
                    self.book_repo.update_book(&book);
                }
            }
        }
        Ok(())
    }
}

pub fn create_job(
    book_repo: SharedBookRepository,
    series_repo: SharedSeriesRepository,
    prompt: SharedPrompt,
) -> Job<Book, SeriesMappingResult> {
    let reader = UnorganizedBookReader::new(book_repo.clone());

    let series_mapping_processor = SeriesMappingProcessor::new(series_repo.clone(), prompt.clone());
    let series_similar_processor = BelongToSeriesProcessor::new(book_repo.clone(), prompt.clone());

    let processor = ProcessorChain::new(Box::new(series_mapping_processor), Box::new(series_similar_processor));

    let writer = SeriesWriter::new(series_repo.clone(), book_repo.clone());

    let mut job = job_builder()
        .reader(Box::new(reader))
        .processor(Box::new(processor))
        .writer(Box::new(writer))
        .build();
    job.chunk_size = 1;

    job
}

fn retrieve_nlgo_set_isbn(book: &Book) -> Option<String> {
    let dict = nlgo::load_raw_key_dict();
    raw_utils::retrieve_series_id_from_raw(&dict, book.originals().get(&Site::NLGO)?)
}

fn convert_book_to_normalize_request(book: &Book) -> NormalizeRequest {
    let mut request = NormalizeRequest::new(book.title());
    let original = book.originals();

    let mut sale_info_vec = Vec::new();
    for (site, raw) in original {
        let dict = raw_utils::load_site_dict(site);
        if let Some(title) = raw_utils::retrieve_title_from_raw(&dict, raw) {
            let mut sale_info = NormalizeRequestSaleInfo::new(&site.to_string(), &title);
            sale_info.price = raw_utils::retrieve_sale_price_from_raw(&dict, raw);
            sale_info.desc = raw_utils::retrieve_description_from_raw(&dict, raw);
            sale_info.series = raw_utils::retrieve_series_list_titles_from_raw(&dict, raw);
            sale_info_vec.push(sale_info);
        }
    }

    if !sale_info_vec.is_empty() {
        request.sale_info = Some(sale_info_vec);
    }

    request
}

fn convert_series_similar_request_book_info(book: &Book) -> SeriesSimilarRequestBookInfo {
    let author = book.originals().iter()
        .find_map(|(site, raw)| {
            let dict = raw_utils::load_site_dict(site);
            dict.get(&RawDataKind::Author)
                .map(|k| raw.get(k))
                .flatten()
                .map(|v| v.to_string())
        });

    SeriesSimilarRequestBookInfo {
        title: book.title().to_owned(),
        publisher: book.publisher_id(),
        author,
    }
}