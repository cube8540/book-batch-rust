use regex::Regex;
use scraper::selector::CssLocalName;
use scraper::{CaseSensitivity, Element, Html, Selector};

pub fn retrieve_item_id(doc: &Html) -> Option<String> {
    let selector = Selector::parse("meta[property=\"eg:itemId\"]").unwrap();
    
    doc.select(&selector)
        .next()
        .map(|e| e.attr("content").unwrap().to_owned())
}

pub fn retrieve_isbn(doc: &Html) -> Option<String> {
    let selector = Selector::parse("meta[property=\"books:isbn\"]").unwrap();

    doc.select(&selector)
        .next()
        .map(|e| e.attr("content").unwrap().to_owned())
}

pub fn retrieve_title(doc: &Html) -> Option<String> {
    let selector = Selector::parse("#contents .prod_title").unwrap();
    doc.select(&selector).next()
        .map(|e| {
            e.text().collect::<Vec<_>>().join(" ")
        })
}

pub fn retrieve_thumbnail(doc: &Html) -> Option<String> {
    let selector = Selector::parse("#contents .portrait_img_box img").unwrap();
    doc.select(&selector).next()
        .map(|e| {
            e.attr("src").map(|s| s.to_owned())
        })?
}

pub fn retrieve_desc_img(doc: &Html) -> Option<String> {
    let selector = Selector::parse("#scrollSpyProdInfo .product_detail_area.detail_img img").unwrap();
    doc.select(&selector).next()
        .map(|e| {
            e.attr("src").map(|s| s.to_owned())
        })?
}

pub fn retrieve_prod_desc(doc: &Html) -> Option<String> {
    let selector = Selector::parse("#scrollSpyProdInfo .product_detail_area.book_intro .info_text").unwrap();
    let mut elements = doc.select(&selector);

    let mut result = Vec::new();
    while let Some(e) = elements.next() {
        result.push(e.inner_html());
    }

    if result.len() > 0 {
       Some(result.join(" "))
    } else {
        None
    }
}

pub fn retrieve_price(doc: &Html) -> (Option<usize>, Option<usize>) {
    let selector = Selector::parse(".prod_price_box .val").unwrap();
    let mut elements = doc.select(&selector);

    let mut sale_price: usize = 0;
    let mut standard_price: usize = 0;

    let sale_price_css = CssLocalName::from("price");
    let standard_price_css = CssLocalName::from("sale_price");

    let regex = Regex::new(r"[^0-9]").unwrap();
    while let Some(e) = elements.next() {
        let parent = e.parent_element().unwrap();
        let value = e.text().collect::<String>();

        let clean = regex.replace_all(&value, "");
        let value = clean.parse::<usize>().unwrap();

        if parent.has_class(&sale_price_css, CaseSensitivity::CaseSensitive) {
            sale_price = value;
        }
        if parent.has_class(&standard_price_css, CaseSensitivity::CaseSensitive) {
            standard_price = value;
        }
    }

    let sale_price = if sale_price > 0 { Some(sale_price) } else { None };
    let standard_price = if standard_price > 0 { Some(standard_price) } else { None };

    (sale_price, standard_price)
}

pub fn retrieve_author(doc: &Html) -> Option<String> {
    let selector = Selector::parse(".product_person .round_gray_box .title_wrap .title_heading").unwrap();
    let mut elements = doc.select(&selector);

    let mut result = Vec::new();
    let empty_text_retex = Regex::new(r"\s*\n\s*").unwrap();
    while let Some(e) = elements.next() {
        let text = e.text()
            .filter(|text| !empty_text_retex.is_match(text))
            .collect::<Vec<_>>()
            .join(":");
        result.push(text);
    }

    if result.len() > 0 {
        Some(result.join(", "))
    } else {
        None
    }
}
