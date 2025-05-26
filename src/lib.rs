use std::fmt;
use std::fmt::Formatter;
use clap::Parser;

pub mod configs;
pub mod provider;
pub mod item;
pub mod batch;

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
    KYOBO
}

impl From<&str> for JobName {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "aladin" => JobName::ALADIN,
            "naver" => JobName::NAVER,
            "nlgo" => JobName::NLGO,
            "kyobo" => JobName::KYOBO,
            _ => panic!("Invalid job name: {}", s),
        }
    }
}

#[derive(Debug, Parser)]
pub struct Argument {

    /// 실행 하려는 배치잡 이름
    ///
    /// # 배치잡 리스트
    /// - `NLGO`: 국립중앙도서관 API를 이용한 도서 데이터 수집
    /// - `NAVER`: 네이버 도서 API를 이용한 도서 데이터 수집
    /// - `ALADIN`: 알라딘 API를 이용한 도서 데이터 수집
    /// - `KYOBO`: 교보문고 파싱을 통한 도서 데이터 수집
    #[arg(short, long)]
    pub job: String,

    /// 수집할 도서의 출판일 검색 시작 날짜
    #[arg(short, long)]
    pub from: Option<String>,

    /// 수집할 도서의 출판일 검색 종료 날짜
    #[arg(short, long)]
    pub to: Option<String>,

    /// 검색할 도서의 출판사 아이디
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

pub fn default_from_date() -> chrono::NaiveDate {
    chrono::Local::now().checked_sub_days(chrono::Days::new(30)).unwrap().date_naive()
}

pub fn default_to_date() -> chrono::NaiveDate {
    chrono::Local::now().checked_add_days(chrono::Days::new(60)).unwrap().date_naive()
}