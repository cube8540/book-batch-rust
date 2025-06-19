pub mod chrome;
mod utils;

use crate::item::{Book, BookBuilder, Raw, RawDataKind, RawKeyDict, RawValue, Site};
use crate::provider::html;
use crate::provider::html::ParsingError;
use reqwest::cookie::Jar;
use reqwest::Url;
use scraper::Html;
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::warn;

const AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.149 Safari/537.36";

const KYOBO_DOMAIN: &'static str = "https://www.kyobobook.co.kr";
const ISBN_SEARCH_ENDPOINT: &'static str = "https://www.kyobobook.co.kr/product/detailViewKor.laf";

/// 교보문고 로그인 제공 트레이트
///
/// # Description
/// 교보문고 로그인과 로그인 후 생성된 쿠키를 관리하고 제공한다.
pub trait LoginProvider {

    type CookieValue: AsRef<str>;

    /// 교보문고 로그인
    ///
    /// # Description
    /// 교보문고에 로그인을 하고 생성된 쿠키를 저장한다.
    fn login(&mut self) -> Result<(), ParsingError>;

    /// 쿠키 반환
    ///
    /// # Description
    /// 로그인 후 생성된 쿠키 리스트를 반환한다.
    fn get_cookies(&self) -> Result<Vec<Self::CookieValue>, ParsingError>;
}

pub struct Client<P>
where
    P: LoginProvider,
{
    login_provider: P,
}

impl <P> Client<P>
where
    P: LoginProvider,
{
    pub fn new(login_provider: P) -> Self {
        Self { login_provider }
    }
}

impl <P> html::Client for Client<P>
where
    P: LoginProvider,
{
    fn get(&self, isbn: &str) -> Result<BookBuilder, ParsingError> {
        let mut url = Url::parse(ISBN_SEARCH_ENDPOINT).unwrap();
        url.query_pairs_mut().append_pair("barcode", isbn);

        let cookie_store = Jar::default();
        let cookies = self.login_provider.get_cookies()?;

        for cookie in cookies {
            cookie_store.add_cookie_str(cookie.as_ref(), &KYOBO_DOMAIN.parse().unwrap());
        }

        let client = reqwest::blocking::Client::builder()
            .cookie_provider(Arc::new(cookie_store))
            .user_agent(AGENT)
            .build()
            .unwrap();

        let request = client.get(url).build().unwrap();
        let response = client
            .execute(request)
            .map_err(|err| ParsingError::RequestFailed(format!("ISBN: {}, ERROR: {:?}", isbn, err)))?;

        let text = response.text().unwrap();
        let parse = html_to_book(&Html::parse_document(&text));

        if let Ok((item_id, mut book_builder)) = parse {
            let series_list = get_series_list(&item_id);
            if let Ok(series_list) = series_list {
                let series = series_list.into_iter()
                    .map(|b| b.to_raw_val())
                    .collect::<Vec<_>>();

                book_builder = book_builder.add_original_raw(Site::KyoboBook, "series", RawValue::Array(series));
                Ok(book_builder)
            } else {
                warn!("Failed to get series list: {}({})", item_id, isbn);
                Ok(book_builder)
            }
        } else {
            Err(parse.unwrap_err())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KyoboResponse {
    pub data: Option<KyoboData>,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KyoboData {
    pub favorite: bool,
    #[serde(rename = "rprsSaleCmdtId")]
    pub rprs_sale_cmdt_id: String,
    #[serde(rename = "rprsSaleCmdtGrpDvsnCode")]
    pub rprs_sale_cmdt_grp_dvsn_code: String,
    #[serde(rename = "rprsSaleCmdtDvsnCode")]
    pub rprs_sale_cmdt_dvsn_code: String,
    pub list: Vec<BookItem>,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookItem {
    #[serde(rename = "totalCount")]
    pub total_count: i32,
    #[serde(rename = "saleCmdtId")]
    pub sale_cmdt_id: String,
    #[serde(rename = "saleCmdtGrpDvsnCode")]
    pub sale_cmdt_grp_dvsn_code: String,
    #[serde(rename = "saleCmdtDvsnCode")]
    pub sale_cmdt_dvsn_code: String,
    #[serde(rename = "saleCmdtClstCode")]
    pub sale_cmdt_clst_code: String,
    #[serde(rename = "cmdtCode")]
    pub cmdt_code: String,
    #[serde(rename = "saleLmttAge")]
    pub sale_lmtt_age: i32,
    pub like: bool,
    pub name: String,
    #[serde(rename = "upntAcmlAmnt")]
    pub upnt_acml_amnt: i32,
    #[serde(rename = "pbcmName")]
    pub pbcm_name: String,
}

impl BookItem {
    pub fn to_raw_val(&self) -> RawValue {
        let mut map: HashMap<String, RawValue> = HashMap::new();
        map.insert("item_id".to_owned(), self.sale_cmdt_id.as_str().into());
        map.insert("isbn".to_owned(), self.cmdt_code.as_str().into());
        map.insert("title".to_owned(), self.name.as_str().into());
        RawValue::Object(map)
    }
}

fn get_series_list(item_id: &str) -> Result<Vec<BookItem>, ParsingError> {
    let url = format!("https://product.kyobobook.co.kr/api/gw/pdt/product/{}/series", item_id);
    let url = Url::parse(&url).unwrap();

    let client = reqwest::blocking::Client::builder()
        .user_agent(AGENT)
        .build()
        .unwrap();

    let response = client
        .get(url)
        .send();
    if response.is_err() {
        return Err(ParsingError::RequestFailed(format!("ERROR: {:?}", response)));
    }
    let response = response.unwrap();
    let text = response.text()
        .map_err(|err| ParsingError::ResponseTextExtractionFailed(format!("ERROR: {:?}", err)))?;

    let response: KyoboResponse = serde_json::from_str(&text)
        .map_err(|err| ParsingError::ResponseTextExtractionFailed(format!("ERROR: {:?}", err)))?;

    if response.status_code != 0 {
        return Err(ParsingError::ItemNotFound);
    }

    let data = response.data.unwrap();
    Ok(data.list)
}

fn html_to_book(document: &Html) -> Result<(String, BookBuilder), ParsingError> {
    let item_id = utils::retrieve_item_id(document)
        .ok_or_else(|| ParsingError::ItemNotFound)?;
    let isbn = utils::retrieve_isbn(document)
        .ok_or_else(|| ParsingError::ItemNotFound)?;
    let title = utils::retrieve_title(document)
        .ok_or_else(|| ParsingError::ElementNotFound("title is not found".to_owned()))?;

    let thumbnail_url = utils::retrieve_thumbnail(document);
    let prod_img_url = utils::retrieve_desc_img(document);
    let prod_desc = utils::retrieve_prod_desc(document);
    let (sale_price, standard_price) = utils::retrieve_price(document);
    let author = utils::retrieve_author(document);

    let mut origin_data = Raw::new();
    origin_data.insert("item_id".to_owned(), item_id.as_str().into());
    origin_data.insert("isbn".to_owned(), isbn.as_str().into());
    origin_data.insert("title".to_owned(), title.as_str().into());
    
    if let Some(s) = thumbnail_url {
        origin_data.insert("thumbnail_url".to_owned(), s.as_str().into());
    }
    if let Some(s) = prod_img_url {
        origin_data.insert("prod_img_url".to_owned(), s.as_str().into());
    }
    if let Some(s) = prod_desc {
        origin_data.insert("prod_description".to_owned(), s.as_str().into());
    }
    if let Some(v) = sale_price {
        origin_data.insert("sale_price".to_owned(), v.into());
    }
    if let Some(v) = standard_price {
        origin_data.insert("standard_price".to_owned(), v.into());
    }
    if let Some(s) = author {
        origin_data.insert("author".to_owned(), s.as_str().into());
    }

    let builder = Book::builder()
        .isbn(isbn.to_owned())
        .title(title.clone())
        .add_original(Site::KyoboBook, origin_data);

    Ok((item_id, builder))
}

pub fn load_raw_key_dict() -> RawKeyDict {
    RawKeyDict::from([
        (RawDataKind::Title, "title".to_owned()),
        (RawDataKind::SalePrice, "sale_price".to_owned()),
        (RawDataKind::Description, "prod_description".to_owned()),
        (RawDataKind::SeriesList, "series".to_owned()),
        (RawDataKind::Author, "author".to_owned()),
    ])
}