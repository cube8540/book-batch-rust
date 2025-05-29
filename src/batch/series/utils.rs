use crate::item::{Book, Raw, RawNumber, RawValue, Site};
use crate::prompt::NormalizeRequestSaleInfo;
use tracing::warn;

/// `RawValue`가 `Text`일 때 저장되어 있는 문자열을 반환한다. 만약 그 외의 값이라면 `None`를 반환한다.
pub fn extract_raw_text(raw: &RawValue) -> Option<String> {
    match raw {
        RawValue::Text(s) => Some(s.clone()),
        _ => {
            warn!("RawValue가 Text 형태가 아닙니다. {}", raw);
            None
        }
    }
}

pub fn extract_raw_u64(raw: &RawValue) -> Option<u64> {
    match raw {
        RawValue::Number(n) => {
            match n {
                RawNumber::UnsignedInt(i) => Some(i.clone()),
                RawNumber::SignedInt(i) => Some(i.clone() as u64),
                RawNumber::Float(_) | RawNumber::Undefined => {
                    warn!("RawValue가 정수 타입이 아닙니다. {}", n);
                    None
                }
            }
        }
        _ => {
            warn!("RawValue가 Number 형태가 아닙니다. {}", raw);
            None
        }
    }
}

/// 도서 정규화 요청에 사용할 판매처별 상품 상세 정보를 설정한다.
pub fn set_sale_info_value(sale_info: &mut NormalizeRequestSaleInfo, site: &Site, raw: &Raw) {
    match site {
        Site::Naver => {
            if let Some(sale_price) = raw.get("discount") {
                sale_info.price = extract_raw_u64(sale_price).map(|v| v as usize);
            }
            if let Some(prod_description) = raw.get("description") {
                sale_info.desc = extract_raw_text(prod_description);
            }
        }
        Site::Aladin => {
            if let Some(sale_price) = raw.get("priceSales") {
                sale_info.price = extract_raw_u64(sale_price).map(|v| v as usize);
            }
            if let Some(prod_description) = raw.get("description") {
                sale_info.desc = extract_raw_text(prod_description);
            }
        }
        Site::KyoboBook => {
            if let Some(sale_price) = raw.get("sale_price") {
                sale_info.price = extract_raw_u64(sale_price).map(|v| v as usize);
            }
            if let Some(prod_description) = raw.get("prod_description") {
                sale_info.desc = extract_raw_text(prod_description);
            }
            if let Some(series) = raw.get("series") {
                let mut s = Vec::new();
                if let RawValue::Array(arr) = series {
                    for e in arr {
                        if let RawValue::Object(map) = e {
                            let title = map.get("title")
                                .map(|v| extract_raw_text(v))
                                .flatten();
                            if title.is_some() {
                                s.push(title.unwrap());
                            }
                        }
                    }
                }
                if !s.is_empty() {
                    sale_info.series = Some(s)
                }
            }
        }
        _ => {}
    }
}

/// `Raw`에서 도서 제목을 나타내는 데이터를 추출한다.
pub fn extract_title_from_raw(site: &Site, raw: &Raw) -> Option<String> {
    let title_key = match site {
        Site::Aladin | Site::Naver | Site::NLGO | Site::KyoboBook => "title",
    };
    let title_raw = raw.get(title_key);
    if let Some(title_raw) = title_raw {
        extract_raw_text(title_raw)
            .map(|s| s.clone())
    } else {
        None
    }
}

/// 도서에서 시리즈 ISBN을 추출한다.
pub fn extract_set_isbn_from_book(book: &Book) -> Option<String> {
    let nlgo_original = book.originals().get(&Site::NLGO);
    if let Some(o) = nlgo_original {
        o.get("set_isbn")
            .map(|v| extract_raw_text(v).map(|s| s.clone()))
            .flatten()
    } else {
        None
    }
}