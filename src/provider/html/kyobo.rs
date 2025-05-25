pub mod chrome;
mod utiles;

use crate::item::{Book, BookBuilder, Raw, Site};
use crate::provider::html;
use crate::provider::html::kyobo::utiles::{retrieve_author, retrieve_desc_img, retrieve_isbn, retrieve_item_id, retrieve_price, retrieve_prod_desc, retrieve_thumbnail, retrieve_title};
use crate::provider::html::ParsingError;
use reqwest::cookie::Jar;
use reqwest::Url;
use scraper::Html;
use std::sync::Arc;

const AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.149 Safari/537.36";

const KYOBO_DOMAIN: &'static str = "https://www.kyobobook.co.kr";
const ISBN_SEARCH_ENDPOINT: &'static str = "https://www.kyobobook.co.kr/product/detailViewKor.laf";

pub trait LoginProvider {
    type CookieValue: AsRef<str>;

    fn login(&mut self) -> Result<(), ParsingError>;
    
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
        html_to_book(&Html::parse_document(&text))
    }
}

fn html_to_book(document: &Html) -> Result<BookBuilder, ParsingError> {
    let item_id = retrieve_item_id(document)
        .ok_or_else(|| ParsingError::ItemNotFound)?;
    let isbn = retrieve_isbn(document)
        .ok_or_else(|| ParsingError::ItemNotFound)?;
    let title = retrieve_title(document)
        .ok_or_else(|| ParsingError::ElementNotFound("title is not found".to_owned()))?;

    let thumbnail_url = retrieve_thumbnail(document);
    let prod_img_url = retrieve_desc_img(document);
    let prod_desc = retrieve_prod_desc(document);
    let (sale_price, standard_price) = retrieve_price(document);
    let author = retrieve_author(document);

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

    Ok(builder)
}