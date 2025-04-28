use serde::Deserialize;
use time::macros::format_description;
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[derive(Debug, Deserialize)]
pub struct Config {
    dir: String,
    name: String,

    /// 최대 로그 파일 개수로 로그 파일이 설정한 개수보다 커질 경우 기존의 로그파일들은 삭제 된다.
    /// 설정 되지 않을 시 로그 파일은 삭제 되지 않는다.
    keep: Option<usize>,

    /// 파일과 stdout에 출력할 로그의 레벨로 지정된 로그 레벨 이상만 로깅된다.
    /// 설정하지 않을시 기본값은 DEBUG로 설정 된다.
    ///
    /// 이 값은 [`tracing::Level`]로 변환 됨으로 자세한 사항은 해당 파일을 확인
    level: Option<String>,

    /// 로깅 파일이 분리 되는 기간으로 .log 파일 하나 당 설정된 기간 동안 로그가 기록 된다.
    /// 설정 되지 않을시 기본값은 DAILY로 설정된다.
    ///
    /// 이 값은 [`rolling::Rotation`]으로 변환 됨으로 자세한 사항은 해당 파일을 확인
    rotation: Option<String>
}

pub fn set_global_logging_config(c: &Config) {
    let mut file_appender = rolling::RollingFileAppender::builder()
        .filename_prefix(c.name.clone())
        .filename_suffix(".log");

    if let Some(rotation) = &c.rotation {
        file_appender = file_appender.rotation(parse_rotation(rotation.as_str()));
    } else {
        file_appender = file_appender.rotation(rolling::Rotation::DAILY);
    }

    if let Some(keep) = c.keep {
        file_appender = file_appender.max_log_files(keep);
    }

    let file_appender = file_appender.build(c.dir.clone()).unwrap();

    let (non_blocking, _) = tracing_appender::non_blocking(file_appender);
    let writer = std::io::stdout.and(non_blocking);

    let mut subscriber = tracing_subscriber::fmt()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_current_span(true)
        .with_span_list(true)
        .with_timer(LocalTime::new(format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]")))
        .with_writer(writer);

    if let Some(level) = &c.level {
        subscriber = subscriber.with_max_level(parse_level(level));
    } else {
        subscriber = subscriber.with_max_level(tracing::Level::DEBUG);
    }

    subscriber.init();
}

fn parse_rotation(s: &str) -> rolling::Rotation {
    match s {
        "DALY" => rolling::Rotation::DAILY,
        "HOURLY" => rolling::Rotation::HOURLY,
        "MINUTELY" => rolling::Rotation::MINUTELY,
        "NAVER" => rolling::Rotation::NEVER,
        _ => panic!("로깅 파일 로테이션(rotation)은 \"{}\", \"{}\", \"{}\", \"{}\"만 가븡 합니다.", "DALY", "HOURLY", "MINUTELY", "NEVER")
    }
}

fn parse_level(l: &str) -> tracing::Level {
    match l {
        "TRACE" => tracing::Level::TRACE,
        "DEBUG" => tracing::Level::DEBUG,
        "INFO" => tracing::Level::INFO,
        "WARN" => tracing::Level::WARN,
        "ERROR" => tracing::Level::ERROR,
        _ => panic!("로그 레벨(level)은 \"{}\", \"{}\", \"{}\", \"{}\", \"{}\"만 가능 합니다.", "TRACE", "DEBUG", "INFO", "WARN", "ERROR")
    }
}