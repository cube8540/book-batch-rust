use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    InvalidBaseUrl,
    RequestFailed(String),
    ResponseTextExtractionFailed(String),
    ResponseParseFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestError {
    MissingRequiredParameter(String), // 필수 매개변수가 누락됨
    InvalidParameter(String),         // 유효하지 않은 매개변수
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredParameter(param) => write!(f, "필수 매개변수가 누락되었습니다: {}", param),
            Self::InvalidParameter(detail) => write!(f, "유효하지 않은 매개변수: {}", detail),
        }
    }
}

impl std::error::Error for RequestError {}