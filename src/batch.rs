pub mod error;
pub mod book;

use crate::batch::error::{JobProcessFailed, JobReadFailed, JobRuntimeError, JobWriteFailed};
use std::collections::HashMap;

pub type JobParameter = HashMap<String, String>;

/// 배치잡 아이템 리더 트레이트 정해진 데이터를 API, 데이터베이스 등 특정 위치에서 조회하거나 검색한다.
/// 현재는 페이징을 지원하지 않기 때문에 잡 1회당 한번만 호출 됨으로 처리에 필요한 데이터들을 모두 로드해야한다.
pub trait Reader {
    type Item;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed>;
}

/// 배치잡 필터 트레이트 정해진 데이터를 `Vec`로 받아 유효한 데이터들만 반환한다.
pub trait Filter {
    type Item;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item>;
}

/// 배치잡 데이터 변환 트레이트 `In` 타입으로 들어온 데이터를 `Out` 타입으로 변경한다.
/// 주로 `Reader`로 읽은 데이터의 변환이 필요하거나, 데이터에 더 많은 정보를 설정하기 위해 사용한다.
pub trait Processor {
    type In;
    type Out;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>>;
}

struct PhantomProcessor<I> {
    _phantom: std::marker::PhantomData<(I, I)>,
}

impl<I> PhantomProcessor<I> {
    pub fn new() -> Self {
        PhantomProcessor {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I> Processor for PhantomProcessor<I> {
    type In = I;
    type Out = I;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
        Ok(item)
    }
}

/// `Reader`, `Filter`, `Processor` 작업 이후 완성된 데이터들을 최종적으로 외부 저장소에 저장하는 트레이트
/// `do_writer` 함수는 여러번 실행 될 수 있으며 각 실행은 독립적인 트랜잭션으로 실행 되어야 한다.
pub trait Writer {
    type Item;

    fn do_write(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>>;
}

pub struct Job<I, O> {
    reader: Box<dyn Reader<Item = I>>,
    filter: Option<Box<dyn Filter<Item = I>>>,
    processor: Box<dyn Processor<In = I, Out = O>>,
    writer: Box<dyn Writer<Item = O>>,
}

impl<I, O> Job<I, O>  {
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

        self.writer.do_write(targets).map_err(|e| JobRuntimeError::WriteFailed(e))?;

        Ok(())
    }
}

pub fn job_builder<I>() -> ReaderBuildStep<I> {
    ReaderBuildStep { reader: None }
}

pub struct ReaderBuildStep<I> {
    reader: Option<Box<dyn Reader<Item = I>>>,
}

impl <I: 'static> ReaderBuildStep<I> {
    pub fn reader(mut self, reader: Box<dyn Reader<Item = I>>) -> Self {
        self.reader = Some(reader);
        self
    }

    pub fn filter(self, filter: Box<dyn Filter<Item = I>>) -> FilterBuildStep<I> {
        if let Some(reader) = self.reader {
            FilterBuildStep { reader, filter: Some(filter), }
        } else {
            panic!("reader is not set");
        }
    }

    pub fn processor<O>(self, processor: Box<dyn Processor<In = I, Out = O>>) -> ProcessorBuildStep<I, O> {
        if let Some(reader) = self.reader {
            ProcessorBuildStep { reader, filter: None, processor }
        } else {
            panic!("reader is not set")
        }
    }

    pub fn writer(self, writer: Box<dyn Writer<Item = I>>) -> WriterBuildStep<I, I> {
        if let Some(reader) = self.reader {
            WriterBuildStep {
                reader,
                filter: None,
                processor: Box::new(PhantomProcessor::new()),
                writer,
            }
        } else {
            panic!("reader is not set")
        }
    }
}

pub struct FilterBuildStep<I> {
    reader: Box<dyn Reader<Item = I>>,
    filter: Option<Box<dyn Filter<Item = I>>>,
}

impl <I: 'static> FilterBuildStep<I> {

    pub fn processor<O>(self, processor: Box<dyn Processor<In = I, Out = O>>) -> ProcessorBuildStep<I, O> {
        ProcessorBuildStep { reader: self.reader, filter: self.filter, processor }
    }

    pub fn writer(self, writer: Box<dyn Writer<Item = I>>) -> WriterBuildStep<I, I> {
        WriterBuildStep {
            reader: self.reader,
            filter: self.filter,
            processor: Box::new(PhantomProcessor::new()),
            writer,
        }
    }
}

pub struct ProcessorBuildStep<I, O> {
    reader: Box<dyn Reader<Item = I>>,
    filter: Option<Box<dyn Filter<Item = I>>>,
    processor: Box<dyn Processor<In = I, Out = O>>,
}

impl <I, O> ProcessorBuildStep<I, O> {

    pub fn writer(self, writer: Box<dyn Writer<Item = O>>) -> WriterBuildStep<I, O> {
        WriterBuildStep { reader: self.reader, filter: self.filter, processor: self.processor, writer }
    }
}

pub struct WriterBuildStep<I, O> {
    reader: Box<dyn Reader<Item = I>>,
    filter: Option<Box<dyn Filter<Item = I>>>,
    processor: Box<dyn Processor<In = I, Out = O>>,
    writer: Box<dyn Writer<Item = O>>,
}

impl <I, O> WriterBuildStep<I, O> {

    pub fn build(self) -> Job<I, O> {
        Job {
            reader: self.reader,
            filter: self.filter,
            processor: self.processor,
            writer: self.writer,
        }
    }
}