use crate::prompt::{Error, NormalizeRequest, Normalized, Prompt, SeriesSimilarRequest};
use reqwest::{blocking, Url};
use serde::{Deserialize, Serialize};
use std::env::var;

const DEFAULT_BRIDGE_HOST: &str = "http://localhost:5000";
const DEFAULT_BRIDGE_NORMALIZE_ENDPOINT: &str = "/normalize";
const DEFAULT_BRIDGE_EMBEDDING_ENDPOINT: &str = "/embedding";
const DEFAULT_BRIDGE_SERIES_SIMILAR_ENDPOINT: &str = "/series-similar";

const DEFAULT_BRIDGE_TIMEOUT: usize = 30000;

/// 브릿지 API 서버 설정 구조체
///
/// # Description
/// 특정 LLM과 연동 되어 있는 서버의 연결 정보를 저장한다.
pub struct BridgeServer {
    /// API 서버의 호스트
    ///
    /// # Note
    /// Host 마지막에 `/`는 입력하지 않는다. (예: http://localhost:8080/ -> http://localhost:8080)
    pub host: String,

    /// 서버 연결 타임아웃 (단위는 밀리세컨드(ms))
    pub timeout: usize,

    /// 도서 제목 정규화 API의 엔드포인트
    pub normalize_endpoint: String,

    /// 텍스트 임베딩 API의 엔드포인트
    pub embedding_endpoint: String,

    /// 시리즈 소속 판단 API의 엔드 포인트
    pub series_similar_endpoint: String
}

impl BridgeServer {
    pub fn new_with_env() -> Self {
        Self {
            host: var("BRIDGE_HOST").unwrap_or_else(|_| DEFAULT_BRIDGE_HOST.to_owned()),
            timeout: var("BRIDGE_TIMEOUT").map(|v| v.parse::<usize>().unwrap()).unwrap_or_else(|_| DEFAULT_BRIDGE_TIMEOUT),
            normalize_endpoint: var("BRIDGE_NORMALIZE_ENDPOINT").unwrap_or_else(|_| DEFAULT_BRIDGE_NORMALIZE_ENDPOINT.to_owned()),
            embedding_endpoint: var("BRIDGE_EMBEDDING_ENDPOINT").unwrap_or_else(|_| DEFAULT_BRIDGE_EMBEDDING_ENDPOINT.to_owned()),
            series_similar_endpoint: var("BRIDGE_SERIES_SIMILAR_ENDPOINT").unwrap_or_else(|_| DEFAULT_BRIDGE_SERIES_SIMILAR_ENDPOINT.to_owned()),
        }
    }
}

/// 임베딩 요청 폼
#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingRequest {
    pub text: Vec<String>,
}

impl EmbeddingRequest {
    pub fn new(text: &[String]) -> Self {
        Self {
            text: text.iter().map(|t| t.clone()).collect()
        }
    }
}

/// 임베딩 결과
///
/// # Description
/// 임베딩 결과와 사용된 텍스트를 저장한다.
#[derive(Debug, Serialize, Deserialize)]
struct Embedding {
    pub encode: Vec<f32>,
    pub original: String,
}

/// 임베딩 응답 형태
#[derive(Debug, Serialize, Deserialize)]
struct Embedded {
    pub embeddings: Vec<Embedding>,
}

/// 시리즈 소속 여부 응답 형태
#[derive(Debug, Serialize, Deserialize)]
struct SeriesSimilar {
    pub result: bool,
    pub reason: Option<String>,
}

/// 브릿지 API 서버 클라이언트
///
/// # Description
/// 특정 LLM과 연동 되어 있는 서버의 API를 호출하는 방식으로 프롬프트 인터페이스를 제공한다.
pub struct BridgeClient {
    server: BridgeServer,
}

impl BridgeClient {
    pub fn new(server: BridgeServer) -> Self {
        Self { server }
    }
}

impl Prompt for BridgeClient {
    fn normalize(&self, request: &NormalizeRequest) -> Result<Normalized, Error> {
        let client = create_blocking_client(&self.server);

        let url = create_request_url(&self.server.host, &self.server.normalize_endpoint);
        let body = serde_json::to_string(request)
            .map_err(|err| Error::ConnectFailed(format!("Failed to serialize request: {}", err)))?;

        let response = client.post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .map_err(|err| Error::ConnectFailed(format!("Failed to send request: {}", err)))?;

        let response_text = response.text()
            .map_err(|err| Error::ResponseParsingFailed(format!("Failed to read response: {}", err)))?;

        let response = serde_json::from_str::<Normalized>(&response_text)
            .map_err(|err| Error::ResponseParsingFailed(format!("Failed to parse response: {}", err)))?;

        Ok(response)
    }

    fn embedding(&self, request: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        let client = create_blocking_client(&self.server);

        let url = create_request_url(&self.server.host, &self.server.embedding_endpoint);
        let body = EmbeddingRequest::new(request);
        let body = serde_json::to_string(&body)
            .map_err(|err| Error::ConnectFailed(format!("Failed to serialize request: {}", err)))?;

        let response = client.post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .map_err(|err| Error::ConnectFailed(format!("Failed to send request: {}", err)))?;

        let response_text = response.text()
            .map_err(|err| Error::ResponseParsingFailed(format!("Failed to read response: {}", err)))?;

        let response = serde_json::from_str::<Embedded>(&response_text)
            .map_err(|err| Error::ResponseParsingFailed(format!("Failed to parse response: {}", err)))?;

        let embeddings = response.embeddings.into_iter()
            .map(|e| e.encode)
            .collect();

        Ok(embeddings)
    }

    fn series_similar(&self, request: &SeriesSimilarRequest) -> Result<bool, Error> {
        let client = create_blocking_client(&self.server);

        let url = create_request_url(&self.server.host, &self.server.series_similar_endpoint);
        let body = serde_json::to_string(request)
            .map_err(|err| Error::ConnectFailed(format!("Failed to serialize request: {}", err)))?;

        let response = client.post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .map_err(|err| Error::ConnectFailed(format!("Failed to send request: {}", err)))?;

        let response_text = response.text()
            .map_err(|err| Error::ResponseParsingFailed(format!("Failed to read response: {}", err)))?;

        let response = serde_json::from_str::<SeriesSimilar>(&response_text)
            .map_err(|err| Error::ResponseParsingFailed(format!("Failed to parse response: {}", err)))?;

        Ok(response.result)
    }
}

fn create_blocking_client(server: &BridgeServer) -> blocking::Client {
    blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(server.timeout as u64))
        .build().unwrap()
}

fn create_request_url(host: &str, endpoint: &str) -> Url {
    let url = format!("{}/{}", host, endpoint);
    Url::parse(&url).unwrap()
}
