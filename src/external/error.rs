#[derive(Debug)]
pub enum ClientError {
    InvalidBaseUrl,
    RequestFailed(String),
    ResponseTextExtractionFailed(String),
    ResponseParseFailed(String),
}