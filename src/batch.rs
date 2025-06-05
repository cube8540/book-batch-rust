pub mod error;
pub mod book;
pub mod series;

use crate::batch::error::{JobProcessFailed, JobReadFailed, JobRuntimeError, JobWriteFailed};
use std::collections::HashMap;

pub type JobParameter = HashMap<String, String>;

/// 배치잡 아이템 리더 트레이트 정해진 데이터를 API, 데이터베이스 등 특정 위치에서 조회하거나 검색한다.
/// 현재는 페이징을 지원하지 않기 때문에 잡 1회당 한번만 호출 됨으로 처리에 필요한 데이터들을 모두 로드해야한다.
///
/// # Type
/// - `Item`: 읽어올 데이터 타입
pub trait Reader {
    type Item;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed>;
}

/// 배치잡 필터 트레이트 정해진 데이터를 `Vec`로 받아 유효한 데이터들만 반환한다.
///
/// # Type
/// - `Item`: 필터링할 데이터 타입
pub trait Filter {
    type Item;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item>;
}

/// 여러 필터들을 하나의 체인으로 결합하는 필터 체인 객체
///
/// # Description
/// 설정된 필터들을 순차적으로 실행하여 하나의 필터 처럼 동작시키며 이전에 실행한 필터의 결과를 다음 필터의 입력값으로 사용한다.
/// 만약 설정된 필터가 없을 경우 최초로 입력 받은 데이터를 그대로 반환한다.
///
/// # Type
/// - `T`: 필터링할 데이터 타입
///
/// # Examples
/// ```
/// use book_batch_rust::batch::{Filter, FilterChain};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct Data {
/// 	id: i32,
/// }
///
/// struct EvenNumberFilter;
/// impl Filter for EvenNumberFilter {
///     type Item = Data;
///
///     fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
///         items.into_iter().filter(|data| data.id % 2 == 0).collect()
///     }
/// }
///
/// struct GraterThen2;
/// impl Filter for GraterThen2 {
///     type Item = Data;
///
///     fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
///         items.into_iter().filter(|data| data.id > 2).collect()
///     }
/// }
///
/// let data_vec = vec![Data { id: 1 }, Data { id: 2 }, Data { id: 3 }, Data { id: 4 }];
///
/// let filter: FilterChain<Data> = FilterChain::new()
///     .add_filter(Box::new(EvenNumberFilter {}))
///     .add_filter(Box::new(GraterThen2 {}));
///
/// let filtered_data_vec = filter.do_filter(data_vec);
/// assert_eq!(filtered_data_vec, vec![Data { id: 4 }]);
/// ```
pub struct FilterChain<T> {
    filters: Vec<Box<dyn Filter<Item = T>>>,
}

impl <T> FilterChain<T> {
    pub fn new() -> Self {
        FilterChain { filters: Vec::new() }
    }

    pub fn add_filter(mut self, filter: Box<dyn Filter<Item = T>>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl <T> Filter for FilterChain<T> {
    type Item = T;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
        if !self.filters.is_empty() {
            self.filters.iter().fold(items, |acc, filter| filter.do_filter(acc))
        } else {
            items
        }
    }
}

/// 배치잡 데이터 변환 트레이트 `In` 타입으로 들어온 데이터를 `Out` 타입으로 변경한다.
/// 주로 `Reader`로 읽은 데이터의 변환이 필요하거나, 데이터에 더 많은 정보를 설정하기 위해 사용한다.
///
/// # Type
/// - `In`: 전달 받을 데이터 타입
/// - `Out`: 반환할 데이터 타입
pub trait Processor {
    type In;
    type Out;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>>;
}

/// 두 개의 프로세서를 하나의 체인으로 결합하는 체인 프로세서 객체
///
/// # Description
/// `first` -> `second` 프로세서를 순서대로 동작시키며 `first`에서 나온 결과를 `second`의 입력 값으로 사용한다.
/// 이를 통해 `I` 타입을 `R` 타입으로 변환한다.
///
/// # Type
/// - `I`: 최초로 입력되는 데이터 타입
/// - `O`: `first`가 반환할 데이터 타입
/// - `R`: 최종적으로 반환될 데이터 타입
///
/// # Examples
/// ```rust
/// use book_batch_rust::batch::error::JobProcessFailed;
/// use book_batch_rust::batch::{Processor, ProcessorChain};
///
/// struct Input {
///     value: i32
/// }
///
/// struct Output {
///     value: i32
/// }
///
/// struct Final {
///     value: i32
/// }
///
/// struct InputToOutput;
/// impl Processor for InputToOutput {
/// 	type In = Input;
///     type Out = Output;
///
///     fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
///         Ok(Output { value: item.value * 2 })
///     }
/// }
///
/// struct OutputToFinal;
/// impl Processor for OutputToFinal {
///     type In = Output;
/// 	type Out = Final;
///
/// 	fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
///         Ok(Final { value: item.value * 3 })
///     }
/// }
///
/// let processor: ProcessorChain<Input, Output, Final> = ProcessorChain::new(Box::new(InputToOutput {}), Box::new(OutputToFinal {}));
/// let final_value = processor.do_process(Input { value: 10 }).unwrap().value;
/// assert_eq!(final_value, 60);
/// ```
pub struct ProcessorChain<I, O, R> {

    /// 최초로 실행될 프로세서
    first: Box<dyn Processor<In = I, Out = O>>,

    /// 최종적으로 실행될 프로세서
    second: Box<dyn Processor<In = O, Out = R>>,
}

impl <I, O, R> ProcessorChain<I, O, R> {
    pub fn new(first: Box<dyn Processor<In = I, Out = O>>, second: Box<dyn Processor<In = O, Out = R>>) -> Self {
        ProcessorChain { first, second }
    }
}

impl <I, O, R> Processor for ProcessorChain<I, O, R> {
    type In = I;
    type Out = R;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>> {
        let first = self.first.do_process(item)?;
        self.second.do_process(first)
            .map_err(|err| JobProcessFailed::new_empty(err.to_string()))
    }
}

/// 입력 타입과 출력 타입이 동일한 프로세서
///
/// # Description
/// `Job`을 구성 시 `Processor`가 필수 컴포넌트지만 데이터 변환이 불필요한 경우 이 프로세서를 사용하여
/// 입력 데이터의 변환 없이 출력한다.
///
/// # Type
/// - `I`: 입/출력 타입
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
///
/// # Type
/// - `Item`: 전달 받을 데이터 타입
pub trait Writer {
    type Item;

    fn do_write(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>>;
}

const DEF_CHUNK_SIZE: usize = 500;

pub struct Job<I, O> {
    reader: Box<dyn Reader<Item = I>>,
    filter: Option<Box<dyn Filter<Item = I>>>,
    processor: Box<dyn Processor<In = I, Out = O>>,
    writer: Box<dyn Writer<Item = O>>,

    /// 청크 사이즈
    ///
    /// # Description
    /// `reader`를 통해 읽은 데이터를 설정된 개수 만큼 나누어 `filter`, `processor`, `writer`의 입력 값으로 사용한다.
    ///
    /// # Note
    /// 이 값이 0 아하로 설정된 상태에서 `run`함수 호출시 패닉이 발생함으로 반드시 1 이상 값으로 설정해야 한다.
    chunk_size: usize,
}

impl<I, O> Job<I, O>  {
    pub fn set_chunk_size(mut self, size: usize) -> Job<I, O> {
        self.chunk_size = size;
        self
    }

    pub fn run(&self, params: &JobParameter) -> Result<(), JobRuntimeError<I, O>> {
        let items = self.reader.do_read(params)
            .map_err(|e| JobRuntimeError::ReadFailed(e))?;

        let items: Vec<I> = if let Some(filter) = &self.filter {
            filter.do_filter(items)
        } else {
            items
        };

        if self.chunk_size == 1 {
            items.into_iter()
                .try_for_each(|item| self.run_task(vec![item]))
        } else {
            chunk_with_owned(items, self.chunk_size).into_iter()
                .try_for_each(|chunk| self.run_task(chunk))
        }
    }

    fn run_task(&self, items: Vec<I>) -> Result<(), JobRuntimeError<I, O>> {
        let mut targets = Vec::new();
        for item in items {
            let target = self.processor.do_process(item)
                .map_err(|e| JobRuntimeError::ProcessFailed(e))?;
            targets.push(target);
        }
        self.writer.do_write(targets).map_err(|e| JobRuntimeError::WriteFailed(e))?;
        Ok(())
    }
}

/// 백터를 지정된 크기의 청크들로 분활 한다.
/// 표준 라이브러리의 [`Vec::chunks`]와 달리 이 함수는 각 청크가 요소들의 소유권을 가지도록 한다.
///
/// # Panic
/// - `size`가 0보다 작거나 같을 경우
///
/// # Example
/// ```
/// use book_batch_rust::batch::chunk_with_owned;
///
/// let vec = vec![1, 2, 3, 4, 5];
/// let chunks = chunk_with_owned(vec, 2);
/// assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
/// ```
pub fn chunk_with_owned<T>(mut vec: Vec<T>, size: usize) -> Vec<Vec<T>> {
    if size <= 0 {
        panic!("size must be greater than 0");
    }

    let mut chunks = Vec::new();
    while vec.len() > 0 {
        let size = std::cmp::min(size, vec.len());
        let chunk = vec.drain(..size).collect::<Vec<_>>();
        chunks.push(chunk);
    }
    chunks
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
            chunk_size: DEF_CHUNK_SIZE,
        }
    }
}