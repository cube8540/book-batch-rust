use std::fmt;
use std::fmt::Formatter;

pub mod configs;
pub mod provider;
pub mod item;
pub mod batch;

#[derive(Debug)]
pub enum ArgumentError {
    InvalidArgument(String),
    InvalidCredentials(String),
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub enum JobName {
    ALADIN,
    NAVER,
    NLGO,
    KYOBO
}

impl JobName {
    pub fn from_str(s: &str) -> Result<Self, ArgumentError> {
        match s {
            "aladin" => Ok(JobName::ALADIN),
            "naver" => Ok(JobName::NAVER),
            "nlgo" => Ok(JobName::NLGO),
            "kyobo" => Ok(JobName::KYOBO),
            _ => Err(ArgumentError::InvalidArgument(format!("Invalid job name: {}", s))),
        }
    }
}

pub struct Argument {
    pub job: JobName,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
}

impl Argument {
    pub fn new(arguments: &[String]) -> Result<Self, ArgumentError> {
        let job_raw = &arguments[1];
        let job_name = JobName::from_str(job_raw)?;

        let arg_len = arguments.len();
        if arg_len == 2 {
            return Ok(Self {
                job: job_name,
                from: None,
                to: None,
            })
        }

        if arg_len < 4 {
            return Err(ArgumentError::InvalidArgument(format!("Invalid argument: {}", arguments.join(" "))));
        }

        let from = chrono::NaiveDate::parse_from_str(&arguments[2], "%Y-%m-%d")
            .map_err(|e| ArgumentError::InvalidArgument(format!("Invalid from date: {}", e)))?;
        let to = chrono::NaiveDate::parse_from_str(&arguments[3], "%Y-%m-%d")
            .map_err(|e| ArgumentError::InvalidArgument(format!("Invalid to date: {}", e)))?;

        Ok(Self {
            job: job_name,
            from: Some(from),
            to: Some(to),
        })
    }
}

pub fn from_to(sub: u64, add: u64) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let current_date = chrono::Local::now();
    let from = current_date
        .checked_sub_days(chrono::Days::new(sub))
        .unwrap()
        .date_naive();
    let to = current_date
        .checked_add_days(chrono::Days::new(add))
        .unwrap()
        .date_naive();
    (from, to)
}