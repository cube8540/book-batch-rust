use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use std::env;
use std::env::VarError;
use mongodb::sync::Client;

mod logging;

/// 실행 환경에 따라 .env 파일을 로드한다.
pub fn load_dotenv() {
    let env_filename = env::var("RUN_MODE")
        .map(|env| format!(".env.{}", env))
        .unwrap_or_else(|_| ".env".into());

    dotenvy::from_filename(env_filename).ok();
}

/// 데이터베이스 연결 풀을 생성한다.
pub fn connect_to_postgres() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

pub fn connect_to_mongo() -> Client {
    let url = env::var("MONGO_URL").expect("MONGO_URL must be set");
    
    Client::with_uri_str(&url).expect("Could not connect to MongoDB")
}

/// 프로그램에서 사용할 로깅 옵션을 설정한다.
pub fn set_global_logging_config() -> Result<(), VarError> {
    let dir = env::var("LOGGER_DIR")?;
    let name = env::var("LOGGER_FILE_NAME")?;

    let keep = env::var("LOGGER_KEEP")
        .map(|v| Some(v.parse::<usize>().unwrap()))
        .unwrap_or_else(|_| None);
    let level = env::var("LOGGER_LEVEL")
        .map(|v| Some(v))
        .unwrap_or_else(|_| None);
    let rotation = env::var("LOGGER_ROTATION")
        .map(|v| Some(v))
        .unwrap_or_else(|_| None);

    let options = logging::Config {
        dir,
        name,
        keep,
        level,
        rotation,
    };

    logging::set_global_logging_config(&options);
    Ok(())
}