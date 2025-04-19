use chrono;

/// 출판사 모델
pub struct Publisher {
    id: u64,
    name: String,
    // API 검색에 사용할 출판사 키워드
    keywords: Vec<String>,
}

impl Publisher {
    fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }
}

/// 도서 시리즈 모델
pub struct Series {
    id: u64,
    name: String,

    isbn: Option<String>,
}

impl Series {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn isbn(&self) -> &Option<String> {
        &self.isbn
    }
}

/// 도서 모델
pub struct Book {
    id: u64,
    isbn: String,
    title: String,

    // 예정된 출판일
    scheduled_pub_date: Option<chrono::NaiveDate>,
    // 실제 출판일
    actual_pub_date: Option<chrono::NaiveDate>,

    series_id: u64,
}

impl Book {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn isbn(&self) -> &str {
        &self.isbn
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn scheduled_pub_date(&self) -> Option<chrono::NaiveDate> {
        self.scheduled_pub_date
    }

    pub fn actual_pub_date(&self) -> Option<chrono::NaiveDate> {
        self.actual_pub_date
    }

    pub fn series_id(&self) -> u64 {
        self.series_id
    }
}