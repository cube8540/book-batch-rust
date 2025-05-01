pub mod filter;
pub mod reader;
pub mod writer;

use crate::book::Publisher;

pub struct Parameter<'job> {
    pub isbn: Option<&'job str>,
    pub publisher: Option<&'job Publisher>,
}