use book_batch_rust::item::repo::{ComposeBookRepository, DieselFilterRepository, DieselPublisherRepository};
use book_batch_rust::item::{SharedBookRepository, SharedFilterRepository, SharedPublisherRepository};
use book_batch_rust::provider::api::{aladin, naver, nlgo};
use book_batch_rust::provider::html::kyobo;
use book_batch_rust::{batch, command_to_parameter, configs, JobName};
use std::rc::Rc;

fn main() {
    configs::load_dotenv();
    configs::set_global_logging_config().expect("Failed to set global logging config");

    let connection = configs::connect_to_postgres();
    let mongo_client = configs::connect_to_mongo();

    let pub_repo = SharedPublisherRepository::new(Box::new(DieselPublisherRepository::new(connection.clone())));
    let book_repo = SharedBookRepository::new(Box::new(ComposeBookRepository::with_origin(connection.clone(), mongo_client.clone())));
    let filter_repo = SharedFilterRepository::new(Box::new(DieselFilterRepository::new(connection.clone())));

    let (job, parameter) = command_to_parameter();
    let job = match job {
        JobName::ALADIN => {
            batch::book::aladin::create_job(
                Rc::new(aladin::Client::new_with_env().unwrap()),
                pub_repo.clone(),
                book_repo.clone(),
                filter_repo.clone(),
            )
        }
        JobName::NAVER => {
            batch::book::naver::create_job(
                Rc::new(naver::Client::new_with_env().unwrap()),
                book_repo.clone(),
            )
        }
        JobName::NLGO => {
            batch::book::nlgo::create_job(
                Rc::new(nlgo::Client::new_with_env().unwrap()),
                pub_repo.clone(),
                book_repo.clone(),
                filter_repo.clone(),
            )
        }
        JobName::KYOBO => {
            batch::book::kyobo::create_job(
                Rc::new(kyobo::Client::new(kyobo::chrome::new_provider().unwrap())),
                Rc::new(kyobo::KyoboAPI::new()),
                book_repo.clone(),
            )
        }
    };
    
    job.run(&parameter).expect("Failed run job");
}
