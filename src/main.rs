use book_batch_rust::batch::JobParameter;
use book_batch_rust::item::repo::{ComposeBookRepository, DieselFilterRepository, DieselPublisherRepository};
use book_batch_rust::provider::api::{aladin, naver, nlgo};
use book_batch_rust::provider::html::kyobo;
use book_batch_rust::{batch, configs, from_to, JobName};
use tracing::error;

fn main() {
    configs::load_dotenv();
    configs::set_global_logging_config().expect("Failed to set global logging config");
    
    let args = std::env::args().collect::<Vec<String>>();
    let args = book_batch_rust::Argument::new(&args).unwrap_or_else(|err| {
        error!("{:?}", err);
        std::process::exit(1);
    });
    
    let connection = configs::connect_to_postgres();
    let mongo_client = configs::connect_to_mongo();

    let (from, to) = if let (Some(from), Some(to)) = (args.from, args.to) {
        (from, to)
    } else {
        from_to(30, 60)
    };

    let publisher_repository = || DieselPublisherRepository::new(connection.clone());
    let book_repository = || ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone());
    let filter_repository = || DieselFilterRepository::new(connection.clone());

    let mut parameter = JobParameter::new();
    parameter.insert(batch::book::PARAM_NAME_FROM_DT.to_owned(), from.format("%Y-%m-%d").to_string());
    parameter.insert(batch::book::PARAM_NAME_TO_DT.to_owned(), to.format("%Y-%m-%d").to_string());

    match args.job {
        JobName::NLGO => {
            let client = || nlgo::Client::new_with_env().unwrap();
            let job = batch::book::nlgo::create_job(
                client,
                publisher_repository,
                book_repository,
                filter_repository,
            );
            job.run(&parameter).expect("Failed run job");
        }
        JobName::ALADIN => {
            let client = || aladin::Client::new_with_env().unwrap();
            let job = batch::book::aladin::create_job(
                client,
                publisher_repository,
                book_repository,
                filter_repository
            );
            job.run(&parameter).expect("Failed run job");
        }
        JobName::NAVER => {
            let client = || naver::Client::new_with_env().unwrap();
            let job = batch::book::naver::create_job(
                client,
                book_repository,
            );
            job.run(&parameter).expect("Failed run job");
        },
        JobName::KYOBO => {
            let client = || kyobo::Client::new(kyobo::chrome::new_provider().unwrap());
            let job = batch::book::kyobo::create_job(
                client,
                book_repository
            );
            job.run(&parameter).expect("Failed run job");
        }
    }

}
