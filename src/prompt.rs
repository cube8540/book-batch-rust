pub mod bridge;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// 프롬프트 사용 중 발생한 에러 열거
#[derive(Debug)]
pub enum Error {
    /// LLM과 연동 중 에러가 발생함
    ConnectFailed(String),

    /// 필수 파라미터 누락
    MissingRequiredParameter(String),

    /// LLM 응답 파싱중 에러가 발생함
    ResponseParsingFailed(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingRequiredParameter(s) => write!(f, "Missing required parameter: {}", s),
            Error::ConnectFailed(s) => write!(f, "Failed to connect to LLM: {}", s),
            Error::ResponseParsingFailed(s) => write!(f, "Failed to parse response: {}", s),
        }
    }
}

/// 제목 정규화 프롬프트의 응답 형태
///
/// # Description
/// 전달 받은 도서 제목에서 불필요한 정보를 제거하고 표준화된 형태로 변환한 결과를 제공한다.
#[derive(Debug, Serialize, Deserialize)]
pub struct Normalized {
    /// 원본 도서 제목 (정규화 이전의 입력값)
    pub original: String,

    /// 정규화된 도서 제목 (불필요한 정보가 제거된 값)
    pub title: String,

    /// 제목에서 제거된 요소에 대한 설명
    pub reason: String
}

/// 도서 판매처별 상세 정보
///
/// # Description
/// 도서 제목 정규화 요청 시 참고할 판매처별 정보를 포함한다.
/// 이 정보들은 더 정확한 정규화를 위해 참고로 사용된다.
#[derive(Debug, Serialize, Deserialize)]
pub struct NormalizeRequestSaleInfo {

    /// 판매 사이트
    ///
    /// # Note
    /// 사이트별로 코드값이 따로 정해져 있지 않아 자연어로 적어도 무관하나
    /// INPUT 토큰의 절약을 위해 사이트의 이니셜 등의 축약어를 넣는 것이 추천된다.
    pub site: String,

    /// 판매처에서 등록된 상품명
    pub title: String,

    /// 판매처에서 등록된 상품가
    pub price: Option<usize>,

    /// 출판사에서 제공하는 도서 상세 설명
    pub desc: Option<String>,

    /// 현재 도서가 속한 시리즈의 다른 도서 제목을 포함하는 리스트
    pub series: Option<Vec<String>>
}

impl NormalizeRequestSaleInfo {

    pub fn new(site: &str, title: &str) -> Self {
        Self {
            site: site.to_owned(),
            title: title.to_owned(),
            price: None,
            desc: None,
            series: None
        }
    }
}

/// 도서 제목 정규화 프롬프트 요청 폼
///
/// # Description
/// 정규화 하고자 하는 도서명과 참고할 수 있는 그 도서의 판매처별 도서 정보를 포함한다.
#[derive(Debug, Serialize, Deserialize)]
pub struct NormalizeRequest {

    /// 정규화 하고자 하는 도서명
    pub title: String,

    /// 판매처별 도서 상세 정보
    pub sale_info: Option<Vec<NormalizeRequestSaleInfo>>
}

impl NormalizeRequest {

    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            sale_info: None
        }
    }
}

/// 시리즈 소속 여부를 검사할 때 활용할 도서 정보
///
/// # Description
/// 시리즈 소속 확인시 참고할 도서의 상세 정보를 저장한다.
#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesSimilarRequestBookInfo {

    /// 도서 제목
    pub title: String,

    /// 출판사 아이디
    pub publisher: u64,

    /// 도서의 저자
    pub author: Option<String>,
}

/// 시리즈 소속 여부 확인 프롬프트 요청 폼
///
/// # Description
/// 기존 시리즈에 속하는 도서 목록과 시리즈에 소속 여부를 확인하고자 하는 신간 정보를 저장한다.
#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesSimilarRequest {

    /// 소속 여부를 확인하고 싶은 신간 도서 정보
    pub new: SeriesSimilarRequestBookInfo,

    /// 기존 시리즈의 도서 목록
    pub series: Vec<SeriesSimilarRequestBookInfo>
}

/// 같은 프롬프트 객체를 여러곳에서 사용 할 수 있도록 하는 [`Rc`] 형태의 공유 프롬프트 타입
pub type SharedPrompt = Rc<Box<dyn Prompt>>;

/// LLM 프롬프트를 제공하는 API 트레이트
///
/// # Description
/// 특정 LLM과 연결하여 도서의 시리즈 자동 분류를 위한 정규화 작업을 하는 인터페이스를 제공한다.
pub trait Prompt {

    /// 입력 받은 도서명을 정규화 하여 표준화된 형태로 반환한다.
    ///
    /// # Parmaeter
    /// - `request`: 정규화할 도서 제목과 참고할 판매처 정보를 담은 요청 객체
    ///
    /// # Returns
    /// - `Normlized`: 정규화된 도서명과 처리 내역을 담은 객체
    fn normalize(&self, request: &NormalizeRequest) -> Result<Normalized, Error>;

    /// 입력 받은 텍스트들을 임베딩 한다.
    ///
    /// # Parameter
    /// - `request`: 임베딩할 텍스트 리스트
    ///
    /// # Returns
    /// 임베딩된 텍스트들을 반환하며 입력된 순서와 동일한 순서로 반환된다.
    ///
    /// # Example
    /// ```
    /// let texts = ["텍스트 1".to_owned(), "텍스트 2".to_owned()];
    /// let embeddings = promp.embedding(&texts);
    ///
    /// // `텍스트 1`의 임베딩 백터
    /// let first_embedding = embeddings[0];
    /// ```
    fn embedding(&self, request: &[String]) -> Result<Vec<Vec<f32>>, Error>;

    /// 입력 받은 신간 정보와 시리즈 목록을 프롬프트에 요청해 신간이 시리즈에 속하는지 여부를 판단한다.
    ///
    /// # Paramter
    /// - request: 신간 정보와 기존 시리즈의 도서 목록 정보를 담은 요청 객체
    ///
    /// # Returns
    /// 신간이 시리즈에 속하는지 여부 (True: 속함/False: 속하지 않음)
    fn series_similar(&self, request: &SeriesSimilarRequest) -> Result<bool, Error>;
}