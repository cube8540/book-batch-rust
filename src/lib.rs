use std::fmt;
use std::fmt::Formatter;
use clap::Parser;
use crate::batch::JobParameter;

pub mod configs;
pub mod provider;
pub mod item;
pub mod batch;
pub mod prompt;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ArgumentError {
    InvalidArgument(String),
    InvalidCredentials(String),
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum JobName {
    ALADIN,
    NAVER,
    NLGO,
    KYOBO,

    SERIES
}

impl From<&str> for JobName {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "aladin" => JobName::ALADIN,
            "naver" => JobName::NAVER,
            "nlgo" => JobName::NLGO,
            "kyobo" => JobName::KYOBO,
            "series" => JobName::SERIES,
            _ => panic!("Invalid job name: {}", s),
        }
    }
}

#[derive(Debug, Parser)]
pub struct Argument {

    /// (Required) 실행 하려는 배치잡 이름
    ///
    /// # Example
    /// ```text
    /// $ cargo run -- --job NLGO
    /// $ cargo run -- -j NLGO
    /// ```
    ///
    /// # Batch Job List
    /// - `NLGO`: 국립중앙도서관 API를 이용한 도서 데이터 수집
    /// - `NAVER`: 네이버 도서 API를 이용한 도서 데이터 수집
    /// - `ALADIN`: 알라딘 API를 이용한 도서 데이터 수집
    /// - `KYOBO`: 교보문고 파싱을 통한 도서 데이터 수집
    /// - `SERIES`: 시리즈가 연결되지 않은 도서들의 적잘한 시리즈를 찾아 연결
    #[arg(short, long)]
    pub job: String,

    /// (Optional) 수집할 도서의 출판일 검색 시작 날짜 (YYYY-MM-DD)
    ///
    /// # Example
    /// ```text
    /// $ cargo run -- --from 2025-01-01
    /// $ cargo run -- -f 2025-01-01
    /// ```
    #[arg(short, long)]
    pub from: Option<String>,

    /// (Optional) 수집할 도서의 출판일 검색 종료 날짜 (YYYY-MM-DD)
    ///
    /// # Example
    /// ```text
    /// $ cargo run -- --to 2025-01-31
    /// $ cargo run -- -t 2025-01-31
    /// ```
    #[arg(short, long)]
    pub to: Option<String>,

    /// (Optional) 검색할 도서의 숫자로 이루어진 출판사 아이디 리스트
    /// 각 출판사 아이디는 공백(" ")으로 구분 한다.
    ///
    /// # Example
    /// ```text
    /// $ cargo run -- --publisher-id 20050726 20110708 20111223
    /// $ cargo run -- -p 20050726 20110708 20111223
    /// ```
    /// ```rust
    /// use clap::Parser;
    /// use book_batch_rust::Argument;
    ///
    /// let argue = Argument::parse();
    /// // [20050726, 20110708, 20111223]
    /// println!("{:?}", argue.publisher_id)
    /// ```
    #[arg(short, long, num_args = 1..)]
    pub publisher_id: Option<Vec<usize>>,
}

impl Argument {

    pub fn get_job(&self) -> JobName {
        self.job.as_str().into()
    }

    pub fn get_from(&self) -> Option<chrono::NaiveDate> {
        self.from.as_ref().map(|from| {
            chrono::NaiveDate::parse_from_str(&from, "%Y-%m-%d").unwrap()
        })
    }

    pub fn get_to(&self) -> Option<chrono::NaiveDate> {
        self.to.as_ref().map(|to| {
            chrono::NaiveDate::parse_from_str(&to, "%Y-%m-%d").unwrap()
        })
    }
}

/// 사용자가 커맨드 라인에 입력한 파라미터들을 `JobParameter`로 만들어 반환한다.
/// `JobParameter`의 키는 각 파라미터의 이름이며, `하이픈(-)`으로 연결된 단어는 `스네이크 케이스(snake_case)`로 변환한다.
///
/// 커맨드 라인 파라미터 중 `--job`은 실행시킬 잡의 이름을 나타내므로 `JobParameter`와 분리하여 튜플의 속성으로 반환한다.
///
/// # Return
/// - `.0`: 실행시킬 배치잡 이름
/// - `.1`: 잡에서 사용될 파라미터
///
/// # Note
/// - `from/to`가 입력 되지 않았을 경우 기본값을 사용하며 `from`은 현재일로 부터 -30일, `to`는 현재일로부터 +60일을 시용한다. (총 90일)
/// - `from`, `to`는 모두 `YYYY-MM-DD` 형식이어야 한다 (ex: 2025-05-01)
/// - `publisher_id`는 콤마(",")로 연결하여 `String` 타입으로 변환한다.(ex: 20050726 20110708 20111223 -> "20050726,20110708,20111223")
pub fn command_to_parameter() -> (JobName, JobParameter) {
    let argument = Argument::parse();

    let mut parameter = JobParameter::new();
    if let Some(from) = argument.get_from().as_ref() {
        parameter.insert("from".to_owned(), from.format("%Y-%m-%d").to_string());
    } else {
        let from = default_from_date();
        parameter.insert("from".to_owned(), from.format("%Y-%m-%d").to_string());
    }

    if let Some(to) = argument.get_to().as_ref() {
        parameter.insert("to".to_owned(), to.format("%Y-%m-%d").to_string());
    } else {
        let to = default_to_date();
        parameter.insert("to".to_owned(), to.format("%Y-%m-%d").to_string());
    }

    if let Some(publisher_id) = argument.publisher_id.as_ref() {
        let mut id_str = String::new();
        for id in publisher_id {
            id_str.push_str(&id.to_string());
            id_str.push(',');
        }
        parameter.insert("publisher_id".to_owned(), id_str);
    }

    (argument.get_job(), parameter)
}

pub fn default_from_date() -> chrono::NaiveDate {
    chrono::Local::now().checked_sub_days(chrono::Days::new(30)).unwrap().date_naive()
}

pub fn default_to_date() -> chrono::NaiveDate {
    chrono::Local::now().checked_add_days(chrono::Days::new(60)).unwrap().date_naive()
}