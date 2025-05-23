pub mod error;
pub mod book;

use crate::batch::error::{JobBuildError, JobProcessFailed, JobReadFailed, JobRuntimeError, JobWriteFailed};
use std::collections::HashMap;

pub type JobParameter = HashMap<String, String>;

pub trait Reader {
    type Item;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed>;
}

pub trait Filter {
    type Item;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item>;
}

pub trait Processor {
    type In;
    type Out;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>>;
}

pub trait Writer {
    type Item;

    fn do_write<I>(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>>;
}

pub struct JobBuilder<I, O, R, F, P, W>
where
    R: Reader<Item = I>,
    F: Filter<Item = I>,
    P: Processor<In = I, Out = O>,
    W: Writer<Item = O>,
{
    reader: Option<R>,
    filter: Option<F>,
    processor: Option<P>,
    writer: Option<W>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O, R, F, P, W> JobBuilder<I, O, R, F, P, W>
where
    R: Reader<Item = I>,
    F: Filter<Item = I>,
    P: Processor<In = I, Out = O>,
    W: Writer<Item = O>,
{
    pub fn new() -> Self {
        JobBuilder {
            reader: None,
            filter: None,
            processor: None,
            writer: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn reader(mut self, reader: R) -> Self {
        self.reader = Some(reader);
        self
    }

    pub fn filter(mut self, filter: F) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn processor(mut self, processor: P) -> Self {
        self.processor = Some(processor);
        self
    }

    pub fn writer(mut self, writer: W) -> Self {
        self.writer = Some(writer);
        self
    }

    pub fn build(self) -> Result<Job<I, O, R, F, P, W>, JobBuildError> {
        let reader = self.reader.ok_or_else(|| JobBuildError::MissingRequireParameter("Reader".to_string()))?;
        let processor = self.processor.ok_or_else(|| JobBuildError::MissingRequireParameter("Processor".to_string()))?;
        let writer = self.writer.ok_or_else(|| JobBuildError::MissingRequireParameter("Writer".to_string()))?;

        Ok(Job {
            reader,
            filter: self.filter,
            processor,
            writer,
        })
    }
}

pub struct Job<I, O, R, F, P, W>
where
    R: Reader<Item = I>,
    F: Filter<Item = I>,
    P: Processor<In = I, Out = O>,
    W: Writer<Item = O>,
{
    reader: R,
    filter: Option<F>,
    processor: P,
    writer: W,
}

impl<I, O, R, F, P, W> Job<I, O, R, F, P, W>
where
    R: Reader<Item = I>,
    F: Filter<Item = I>,
    P: Processor<In = I, Out = O>,
    W: Writer<Item = O>,
{
    pub fn builder() -> JobBuilder<I, O, R, F, P, W> {
        JobBuilder::new()
    }

    pub fn run(&self, params: &JobParameter) -> Result<(), JobRuntimeError<I, O>> {
        let items = self.reader.do_read(params)
            .map_err(|e| JobRuntimeError::ReadFailed(e))?;

        let items: Vec<I> = if let Some(filter) = &self.filter {
            filter.do_filter(items)
        } else {
            items
        };

        let mut targets: Vec<O> = Vec::new();
        for item in items {
            let target = self.processor.do_process(item)
                .map_err(|e| JobRuntimeError::ProcessFailed(e))?;
            targets.push(target);
        }

        self.writer.do_write::<Vec<O>>(targets)
            .map_err(|e| JobRuntimeError::WriteFailed(e))?;

        Ok(())
    }
}