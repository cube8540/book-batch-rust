pub mod error;
pub mod book;

use crate::batch::error::{JobBuildError, JobProcessFailed, JobReadFailed, JobRuntimeError, JobWriteFailed};
use std::collections::HashMap;

pub type JobParameter = HashMap<String, String>;

/// 동일한 타입의 객체를 여러 번 생성할 수 있는 트레이트
///
/// # Description
/// 잡 구성 요소가 동일한 타입의 객체를 필요로 할 때 사용한다.
/// 예를 들어 `Reader`와 `Writer`가 동일한 저장소 객체를 필요로 하는 경우 `Provider`를 통해 각각의 컴포넌트에 새로운 객체를 제공한다.
///
/// # Example
/// ```
/// use book_batch_rust::batch::{Job, JobParameter, PhantomFilter, PhantomProcessor, Provider, Reader, Writer};
/// use book_batch_rust::batch::error::{JobReadFailed, JobWriteFailed};
///
///
/// #[derive(PartialEq, Eq, Debug)]
/// struct Store {
///     v: String
/// }
///
/// struct StoreReader {
///     store: Store // Writer와 같은 타입의 객체를 요구
/// }
///
/// impl Reader for StoreReader {
///     type Item = String;
///
///     fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed> {
///         Ok(vec![self.store.v.clone()])
///     }
/// }
///
/// struct StoreWriter {
///     store: Store // Reader와 같은 타입의 객체를 요구
/// }
///
/// impl Writer for StoreWriter {
///     type Item = String;
///
///     fn do_write<I>(&self, items: Vec<Self::Item>) -> Result<(), JobWriteFailed<Self::Item>> {
///         println!("{:?}", items);
///         Ok(())
///     }
/// }
///
/// fn create_job_without_provider(store_1: Store, store_2: Store) -> Job<String, String, StoreReader, PhantomFilter<String>, PhantomProcessor<String>, StoreWriter> {
///     Job::builder()
///         .reader(StoreReader { store: store_1 })
///         .writer(StoreWriter { store: store_2 })
///         .filter(PhantomFilter::new())
///         .processor(PhantomProcessor::new())
///         .build()
///         .unwrap()
/// }
///
/// // Reader, Writer가 TypeA 타입의 객체 대한 소유권을 요구하여 함수에서 같은 타입의 객체를 두 개 받는다.
/// let store_1 = Store { v: "str".to_owned() };
/// let store_2 = Store { v: "str".to_owned() };
/// let job = create_job_without_provider(store_1, store_2);
///
/// fn create_job_with_provider(p: impl Provider<Item=Store>) -> Job<String, String, StoreReader, PhantomFilter<String>, PhantomProcessor<String>, StoreWriter> {
///     Job::builder()
///         .reader(StoreReader { store: p.retrieve() })
///         .writer(StoreWriter { store: p.retrieve() })
///         .filter(PhantomFilter::new())
///         .processor(PhantomProcessor::new())
///         .build()
///         .unwrap()
/// }
///
/// // Reader, Writer가 같은 TypeA 타입의 객체 대한 소유권을 요구하지만, Provider 하나만 사용하여 함수의 문법이 간결해짐.
/// let job = create_job_with_provider(|| Store { v: "str".to_owned() });
/// ```
/// # Note
/// [`Rc`], [`Arc`] 등 다른 해결 방식이 더 효율적일 수 있으니 반드시 확인하고 사용할 것.
pub trait Provider {

    type Item;

    fn retrieve(&self) -> Self::Item;
}

impl<T, O> Provider for T where T: Fn() -> O {

    type Item = O;

    fn retrieve(&self) -> Self::Item {
        self()
    }
}

pub trait Reader {
    type Item;

    fn do_read(&self, params: &JobParameter) -> Result<Vec<Self::Item>, JobReadFailed>;
}

pub trait Filter {
    type Item;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item>;
}

pub struct PhantomFilter<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PhantomFilter<T> {
    pub fn new() -> Self {
        PhantomFilter {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Filter for PhantomFilter<T> {
    type Item = T;

    fn do_filter(&self, items: Vec<Self::Item>) -> Vec<Self::Item> {
        items
    }
}

pub trait Processor {
    type In;
    type Out;

    fn do_process(&self, item: Self::In) -> Result<Self::Out, JobProcessFailed<Self::In>>;
}

pub struct PhantomProcessor<I> {
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