use crate::procedure::filter::Filter;
use crate::procedure::reader::Reader;
use crate::procedure::writer::Writer;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

pub mod book;
pub mod config;
pub mod provider;
pub mod procedure;

#[derive(Debug)]
pub enum ArgumentError {
    InvalidArgument(String),
}

pub enum JobName {
    ALADIN,
    NAVER,
    NLGO,
}

impl JobName {
    pub fn from_str(s: &str) -> Result<Self, ArgumentError> {
        match s {
            "aladin" => Ok(JobName::ALADIN),
            "naver" => Ok(JobName::NAVER),
            "nlgo" => Ok(JobName::NLGO),
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



pub fn create_nlgo_job_attr(
    nlgo: &config::api::Credentials,
    connection: &Pool<ConnectionManager<PgConnection>>
) -> (impl Reader, impl Writer, impl Filter) {
    let client = provider::api::nlgo::Client::new(nlgo.key());
    let nlgo_reader = procedure::reader::nlgo::new(client);
    let writer = procedure::writer::NewBookOnlyWriter::new(
        book::repository::diesel::book::new(connection.clone())
    );
    let filter_chain = create_filter_chain(&connection);

    (nlgo_reader, writer, filter_chain)
}

pub fn create_aladin_job_attr(
    aladin: &config::api::Credentials,
    connection: &Pool<ConnectionManager<PgConnection>>
) -> (impl Reader, impl Writer, impl Filter) {
    let client = provider::api::aladin::Client::new(aladin.key());
    let aladin_reader = procedure::reader::aladin::new(client);
    let writer = procedure::writer::UpsertBookWriter::new(
        book::repository::diesel::book::new(connection.clone())
    );
    let filter_chain = create_filter_chain(&connection);

    (aladin_reader, writer, filter_chain)
}

pub fn create_naver_job_attr(
    naver: &config::api::Credentials,
    connection: &Pool<ConnectionManager<PgConnection>>
) -> (impl Reader, impl Writer) {
    let (key, secret) = (naver.key(), naver.secret());
    let client = provider::api::naver::new(key.to_owned(), secret.unwrap().to_owned());
    let naver_reader = procedure::reader::naver::new(
        client,
        book::repository::diesel::book::new(connection.clone())
    );
    let writer = procedure::writer::UpsertBookWriter::new(
        book::repository::diesel::book::new(connection.clone())
    );

    (naver_reader, writer)
}

fn create_filter_chain(connection: &Pool<ConnectionManager<PgConnection>>) -> procedure::filter::FilterChain {
    let empty_isbn_filter = procedure::filter::EmptyIsbnFilter {};
    let origin_data_filter = procedure::filter::OriginDataFilter::new(
        book::repository::diesel::book_origin_filter::new(connection.clone())
    );
    let mut filter_chain = procedure::filter::FilterChain::new();
    filter_chain.add_filter(Box::new(empty_isbn_filter));
    filter_chain.add_filter(Box::new(origin_data_filter));

    filter_chain
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