#[derive(Debug)]
pub enum JobRuntimeError<I, O> {
    ReadFailed(JobReadFailed),
    ProcessFailed(JobProcessFailed<I>),
    WriteFailed(JobWriteFailed<O>),
}

#[derive(Debug)]
pub enum JobBuildError {
    MissingRequireParameter(String),
}

#[derive(Debug)]
pub enum JobReadFailed {
    EmptyData(String),
    InvalidArguments(String),
    UnknownError(String),
}

impl std::fmt::Display for JobReadFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobReadFailed::EmptyData(message) => write!(f, "Empty data, {}", message.to_owned()),
            JobReadFailed::InvalidArguments(message) => write!(f, "Invalid arguments, {}", message.to_owned()),
            JobReadFailed::UnknownError(message) => write!(f, "Unknown, {}", message.to_owned()),
        }
    }
}

impl std::error::Error for JobReadFailed {}

pub struct JobProcessFailed<I> {
    item: Option<I>,
    message: String,
}

impl <I> JobProcessFailed<I> {

    pub fn new(item: I, message: String) -> Self {
        JobProcessFailed {
            item: Some(item),
            message,
        }
    }

    pub fn new_empty(message: String) -> Self {
        JobProcessFailed {
            item: None,
            message,
        }
    }

    pub fn item(&self) -> &Option<I> {
        &self.item
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl <I> std::fmt::Display for JobProcessFailed<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<I> std::fmt::Debug for JobProcessFailed<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl <I> std::error::Error for JobProcessFailed<I> {}

pub struct JobWriteFailed<O> {
    item: Vec<O>,
    message: String,
}

impl<O> JobWriteFailed<O> {
    pub fn new(item: Vec<O>, message: &str) -> Self {
        JobWriteFailed {
            item,
            message: message.to_owned(),
        }
    }

    pub fn item(&self) -> &Vec<O> {
        &self.item
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl<O> std::fmt::Display for JobWriteFailed<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<O> std::fmt::Debug for JobWriteFailed<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<O> std::error::Error for JobWriteFailed<O> {}
