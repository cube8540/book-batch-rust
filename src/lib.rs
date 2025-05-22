use crate::item::repo::{ComposeBookRepository, DieselFilterRepository};
use crate::item::Site;
use crate::procedure::filter::Filter;
use crate::procedure::reader::Reader;
use crate::procedure::writer::Writer;
use crate::provider::html::kyobo::LoginProvider;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use std::fmt;
use std::fmt::Formatter;

pub mod procedure;
pub mod configs;
pub mod provider;
pub mod item;

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



pub fn create_nlgo_job_attr(
    connection: Pool<ConnectionManager<PgConnection>>,
    mongo_client: mongodb::sync::Client
) -> (impl Reader, impl Writer, impl Filter) {
    let client = provider::api::nlgo::new_client()
        .expect("Failed to create nlgo client");
    let nlgo_reader = procedure::reader::nlgo::new(client);
    let writer = procedure::writer::NewBookOnlyWriter::new(
        ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())
    );
    let filter_chain = create_filter_chain(connection.clone(), Site::NLGO);

    (nlgo_reader, writer, filter_chain)
}

pub fn create_aladin_job_attr(
    connection: Pool<ConnectionManager<PgConnection>>,
    mongo_client: mongodb::sync::Client
) -> (impl Reader, impl Writer, impl Filter) {
    let client = provider::api::aladin::new_client()
        .expect("Failed to create aladin client");
    let aladin_reader = procedure::reader::aladin::new(client);
    let writer = procedure::writer::UpsertBookWriter::new(
        ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())
    );
    let filter_chain = create_filter_chain(connection.clone(), Site::Aladin);

    (aladin_reader, writer, filter_chain)
}

pub fn create_naver_job_attr(
    connection: Pool<ConnectionManager<PgConnection>>,
    mongo_client: mongodb::sync::Client
) -> (impl Reader, impl Writer) {
    let client = provider::api::naver::new_client()
        .expect("Failed to create naver client");
    let naver_reader = procedure::reader::naver::new(
        client,
        ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())
    );
    let writer = procedure::writer::UpsertBookWriter::new(
        ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())
    );

    (naver_reader, writer)
}

pub fn create_kyobo_job_attr(
    connection: Pool<ConnectionManager<PgConnection>>,
    mongo_client: mongodb::sync::Client
) -> Result<(impl Reader, impl Writer), ArgumentError> {
    let mut login_provider = provider::html::kyobo::chrome::new_provider()
        .expect("Failed to create kyobo login provider");

    _ = login_provider.login()
        .map_err(|err| ArgumentError::InvalidCredentials(err.to_string()))?;

    let client = provider::html::kyobo::Client::new(login_provider);
    let kyobo_reader = procedure::reader::kyobo::KyoboReader::new(
        client,
        ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())
    );
    let writer = procedure::writer::UpsertBookWriter::new(
        ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())
    );

    Ok((kyobo_reader, writer))
}

fn create_filter_chain(connection: Pool<ConnectionManager<PgConnection>>, site: Site) -> procedure::filter::FilterChain {
    let empty_isbn_filter = procedure::filter::EmptyIsbnFilter {};
    let origin_data_filter = procedure::filter::OriginDataFilter::new(
        DieselFilterRepository::new(connection.clone()),
        site
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