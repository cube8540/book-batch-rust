pub mod filter;
pub mod reader;
mod writer;

use crate::book::{Publisher, Site};

pub struct Parameter<'job> {
    pub isbn: Option<&'job str>,
    pub publisher: Option<&'job Publisher>,
    pub site: Site,
}