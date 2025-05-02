pub mod filter;
pub mod reader;
pub mod writer;

use crate::book::Publisher;

pub struct Parameter<'job> {
    pub isbn: &'job [&'job str],
    pub publisher: Option<&'job Publisher>,
    pub from: Option<&'job chrono::NaiveDate>,
    pub to: Option<&'job chrono::NaiveDate>,
}