use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    MissingRequiredParameter(String), // 필수 매개변수가 누락됨
    InvalidBaseUrl,
    RequestFailed(String),
    ResponseTextExtractionFailed(String),
    ResponseParseFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestError {
    InvalidParameter(String),         // 유효하지 않은 매개변수
}