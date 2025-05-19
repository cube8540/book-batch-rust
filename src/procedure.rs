use crate::book::{Book, Publisher};
use chrono::NaiveDate;

pub mod reader;
pub mod filter;
pub mod writer;

pub struct Parameter {
    publisher: Option<Publisher>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

pub struct ParameterBuilder {
    publisher: Option<Publisher>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

impl ParameterBuilder {
    pub fn new() -> Self {
        ParameterBuilder {
            publisher: None,
            from: None,
            to: None,
        }
    }

    pub fn publisher(mut self, publisher: Publisher) -> Self {
        self.publisher = Some(publisher);
        self
    }

    pub fn from(mut self, from: NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    pub fn build(self) -> Parameter {
        Parameter {
            publisher: self.publisher,
            from: self.from,
            to: self.to,
        }
    }
}

impl Parameter {
    pub fn builder() -> ParameterBuilder {
        ParameterBuilder::new()
    }

    pub fn publisher(&self) -> &Option<Publisher> {
        &self.publisher
    }

    pub fn from(&self) -> &Option<NaiveDate> {
        &self.from
    }

    pub fn to(&self) -> &Option<NaiveDate> {
        &self.to
    }
}


pub struct Job {
    reader: Box<dyn reader::Reader>,
    filter: Option<Box<dyn filter::Filter>>,
    writer: Box<dyn writer::Writer>,
}

pub struct JobBuilder {
    reader: Option<Box<dyn reader::Reader>>,
    filter: Option<Box<dyn filter::Filter>>,
    writer: Option<Box<dyn writer::Writer>>,
}

impl JobBuilder {
    pub fn new() -> Self {
        JobBuilder {
            reader: None,
            filter: None,
            writer: None,
        }
    }

    pub fn reader(mut self, reader: Box<dyn reader::Reader>) -> Self {
        self.reader = Some(reader);
        self
    }

    pub fn filter(mut self, filter: Box<dyn filter::Filter>) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn writer(mut self, writer: Box<dyn writer::Writer>) -> Self {
        self.writer = Some(writer);
        self
    }

    pub fn build(self) -> Result<Job, &'static str> {
        let reader = self.reader.ok_or("Reader는 필수 값입니다")?;
        let writer = self.writer.ok_or("Writer는 필수 값입니다")?;

        Ok(Job {
            reader,
            filter: self.filter,
            writer,
        })
    }
}

impl Job {
    pub fn builder() -> JobBuilder {
        JobBuilder::new()
    }

    pub fn run(&self, parameter: &Parameter) -> Vec<Book> {
        let books = self.reader.read_books(parameter);

        let filtered_book = if let Some(filter) = &self.filter {
            filter.do_filter(books)
        } else {
            books
        };

        self.writer.write(&filtered_book)
    }
}