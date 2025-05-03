pub mod filter;
pub mod reader;
pub mod writer;

use crate::book::{Book, Publisher};

pub struct Parameter<'job> {
    pub publisher: Option<&'job Publisher>,
    pub from: Option<&'job chrono::NaiveDate>,
    pub to: Option<&'job chrono::NaiveDate>,
}

pub struct Job<'job> {
    reader: &'job dyn reader::Reader,
    filter: Option<&'job dyn filter::Filter>,
    writer: &'job dyn writer::Writer,
}

pub struct JobBuilder<'job> {
    reader: Option<&'job dyn reader::Reader>,
    filter: Option<&'job dyn filter::Filter>,
    writer: Option<&'job dyn writer::Writer>,
}

impl<'job> JobBuilder<'job> {
    pub fn new() -> Self {
        Self {
            reader: None,
            filter: None,
            writer: None,
        }
    }

    pub fn reader(mut self, reader: &'job dyn reader::Reader) -> Self {
        self.reader = Some(reader);
        self
    }

    pub fn filter(mut self, filter: &'job dyn filter::Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn writer(mut self, writer: &'job dyn writer::Writer) -> Self {
        self.writer = Some(writer);
        self
    }

    pub fn build(self) -> Result<Job<'job>, &'static str> {
        let reader = self.reader.ok_or("Reader is required")?;
        let writer = self.writer.ok_or("Writer is required")?;
        Ok(Job {
            reader,
            filter: self.filter,
            writer,
        })
    }
}

impl<'job> Job<'job> {
    pub fn builder() -> JobBuilder<'job> {
        JobBuilder::new()
    }

    pub fn run(&self, parameter: &Parameter) -> Vec<Book> {
        let books = self.reader.read_books(parameter);
        let books: Vec<&Book> = books.iter().collect();

        let filtered_book = if let Some(filter) = self.filter {
            filter.do_filter(&books)
        } else {
            books
        };

        self.writer.write(&filtered_book)
    }
}
#[derive(Default)]
pub struct ParameterBuilder<'job> {
    publisher: Option<&'job Publisher>,
    from: Option<&'job chrono::NaiveDate>,
    to: Option<&'job chrono::NaiveDate>,
}

impl<'job> ParameterBuilder<'job> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publisher(mut self, publisher: &'job Publisher) -> Self {
        self.publisher = Some(publisher);
        self
    }

    pub fn from(mut self, from: &'job chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: &'job chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    pub fn build(self) -> Parameter<'job> {
        Parameter {
            publisher: self.publisher,
            from: self.from,
            to: self.to,
        }
    }
}

impl<'job> Parameter<'job> {
    pub fn builder() -> ParameterBuilder<'job> {
        ParameterBuilder::new()
    }
}