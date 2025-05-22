use book_batch_rust::item::repo::DieselPublisherRepository;
use book_batch_rust::item::PublisherRepository;
use book_batch_rust::{configs, create_aladin_job_attr, create_kyobo_job_attr, create_naver_job_attr, create_nlgo_job_attr, from_to, procedure, JobName};
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
    
    let publisher_repository = DieselPublisherRepository::new(connection.clone());
    match args.job {
        JobName::NLGO => {
            let (reader, writer, filter) =
                create_nlgo_job_attr(connection.clone(), mongo_client.clone());
            let job = procedure::Job::builder()
                .reader(Box::new(reader))
                .writer(Box::new(writer))
                .filter(Box::new(filter))
                .build()
                .unwrap();
    
            let publishers = publisher_repository.get_all();
            for publisher in publishers {
                let parameter = procedure::Parameter::builder()
                    .from(from)
                    .to(to)
                    .publisher(publisher);
                job.run(&parameter.build());
            }
        }
        JobName::ALADIN => {
            let (reader, writer, filter) =
                create_aladin_job_attr(connection.clone(), mongo_client.clone());
            let job = procedure::Job::builder()
                .reader(Box::new(reader))
                .writer(Box::new(writer))
                .filter(Box::new(filter))
                .build()
                .unwrap();
    
            let publishers = publisher_repository.get_all();
            for publisher in publishers {
                let parameter = procedure::Parameter::builder()
                    .publisher(publisher);
                job.run(&parameter.build());
            }
        }
        JobName::NAVER => {
            let (reader, writer) = create_naver_job_attr(connection.clone(), mongo_client.clone());
            let job = procedure::Job::builder()
                .reader(Box::new(reader))
                .writer(Box::new(writer))
                .build()
                .unwrap();
    
            let parameter = procedure::Parameter::builder()
                .from(from)
                .to(to);
            job.run(&parameter.build());
        },
        JobName::KYOBO => {
            let (reader, writer) = create_kyobo_job_attr(connection.clone(), mongo_client.clone())
                .unwrap_or_else(|err| {
                    error!("{:?}", err);
                    std::process::exit(1);
                });
            let job = procedure::Job::builder()
                .reader(Box::new(reader))
                .writer(Box::new(writer))
                .build()
                .unwrap();
            let parameter = procedure::Parameter::builder()
                .from(from)
                .to(to);
            job.run(&parameter.build());
        }
    }

}
