use crate::item::{Raw, RawDataKind, RawKeyDict, RawValue, Site};
use crate::provider::api::{aladin, naver, nlgo};
use crate::provider::html::kyobo;
use tracing::warn;

pub fn load_site_dict(site: &Site) -> RawKeyDict {
    match site {
        Site::NLGO => nlgo::load_raw_key_dict(),
        Site::Naver => naver::load_raw_key_dict(),
        Site::Aladin => aladin::load_raw_key_dict(),
        Site::KyoboBook => kyobo::load_raw_key_dict(),
    }
}

pub fn retrieve_title_from_raw(dict: &RawKeyDict, raw: &Raw) -> Option<String> {
    let key = dict.get(&RawDataKind::Title)?;
    let opt = raw.get(key).map(|v| String::from(v));
    if opt.is_some() && !opt.as_ref().unwrap().is_empty() {
        opt
    } else {
        None
    }
}

pub fn retrieve_series_id_from_raw(dict: &RawKeyDict, raw: &Raw) -> Option<String> {
    let key = dict.get(&RawDataKind::SeriesID)?;
    let opt = raw.get(key).map(|v| String::from(v));
    if opt.is_some() && !opt.as_ref().unwrap().is_empty() {
        opt
    } else {
        None
    }
}

pub fn retrieve_description_from_raw(dict: &RawKeyDict, raw: &Raw) -> Option<String> {
    let key = dict.get(&RawDataKind::Description)?;
    let opt = raw.get(key).map(|v| String::from(v));
    if opt.is_some() && !opt.as_ref().unwrap().is_empty() {
        opt
    } else {
        None
    }
}

pub fn retrieve_sale_price_from_raw(dict: &RawKeyDict, raw: &Raw) -> Option<usize> {
    let key = dict.get(&RawDataKind::SalePrice)?;

    raw.get(key)
        .and_then(|v| {
            usize::try_from(v)
                .inspect_err(|e| {
                    warn!("Failed to parse sale price: {} (Err ==> {})", v, e);
                })
                .ok()
        })
}

pub fn retrieve_series_list_titles_from_raw(dict: &RawKeyDict, raw: &Raw) -> Option<Vec<String>> {
    let key = dict.get(&RawDataKind::SeriesList)?;
    let raw = raw.get(key)?;

    retrieve_series_list_titles_from_raw_value(dict, raw)
}

fn retrieve_series_list_titles_from_raw_value(dict: &RawKeyDict, raw: &RawValue) -> Option<Vec<String>> {
    match raw {
        RawValue::Null | RawValue::Number(_) | RawValue::Bool(_) => None,
        RawValue::Text(s) => Some(vec![s.to_owned()]),
        RawValue::Object(o) => retrieve_title_from_raw(dict, o).map(|t| vec![t]),
        RawValue::Array(arr) => {
            let results = arr.iter()
                .filter_map(|v| retrieve_series_list_titles_from_raw_value(dict, v))
                .flatten()
                .collect::<Vec<_>>();
            if !results.is_empty() {
                Some(results)
            } else {
                None
            }
        }
    }
}