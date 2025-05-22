pub mod chrome;

use crate::item::{Book, BookBuilder, Site};
use crate::provider::html;
use crate::provider::html::ParsingError;
use reqwest::cookie::Jar;
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;

const AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.149 Safari/537.36";

const KYOBO_DOMAIN: &'static str = "https://www.kyobobook.co.kr";
const ISBN_SEARCH_ENDPOINT: &'static str = "https://www.kyobobook.co.kr/product/detailViewKor.laf";

type CookieValue = String;
pub trait LoginProvider {
    fn login(&mut self) -> Result<(), ParsingError>;
    
    fn get_cookies(&self) -> Result<Vec<CookieValue>, ParsingError>;
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
            cookie_store.add_cookie_str(&cookie, &KYOBO_DOMAIN.parse().unwrap());
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
    let isbn_meta_selector = Selector::parse("meta[property=\"books:isbn\"]").unwrap();
    let isbn_meta_element = document.select(&isbn_meta_selector).next();

    if isbn_meta_element.is_none() {
        return Err(ParsingError::ItemNotFound)
    }
    let isbn = isbn_meta_element.unwrap().attr("content").unwrap();

    let title_selector = Selector::parse(".prod_title").unwrap();
    let title_element = document.select(&title_selector).next();
    if title_element.is_none() {
        return Err(ParsingError::ElementNotFound(".prod_title is not found".to_owned()));
    }
    let title = title_element.unwrap().inner_html();

    let thumbnail_selector = Selector::parse("#contents .prod_detail_header .prod_detail_view_wrap .prod_detail_view_area .thumb .portrait_img_box img").unwrap();
    let thumbnail_element = document.select(&thumbnail_selector).next();
    let thumbnail_url = thumbnail_element.map(|e| e.attr("src").unwrap().to_owned());

    let prod_img_selector = Selector::parse("#scrollSpyProdInfo .product_detail_area.detail_img img").unwrap();
    let prod_img_element = document.select(&prod_img_selector).next();
    let prod_img_url = prod_img_element.map(|e| e.attr("src").unwrap().to_owned());

    let prod_description_selector = Selector::parse("#scrollSpyProdInfo .product_detail_area.book_intro .intro_bottom .info_text").unwrap();
    let prod_description_element = document.select(&prod_description_selector).next();
    let prod_description = prod_description_element.map(|e| e.inner_html());

    let mut origin_data: HashMap<String, String> = HashMap::new();
    origin_data.insert("isbn".to_owned(), isbn.to_owned());
    origin_data.insert("title".to_owned(), title.clone());
    if let Some(thumbnail_url) = thumbnail_url {
        origin_data.insert("thumbnail_url".to_owned(), thumbnail_url);
    }
    if let Some(prod_img_url) = prod_img_url {
        origin_data.insert("prod_img_url".to_owned(), prod_img_url);
    }
    if let Some(prod_description) = prod_description {
        origin_data.insert("prod_description".to_owned(), prod_description);
    }

    let builder = Book::builder()
        .isbn(isbn.to_owned())
        .title(title.clone())
        .add_original(Site::KyoboBook, origin_data);

    Ok(builder)
}