// @generated automatically by Diesel CLI.
diesel::table! {
    books.book (id) {
        id -> Int8,
        #[max_length = 13]
        isbn -> Varchar,
        #[max_length = 256]
        title -> Varchar,
        publisher_id -> Int8,
        scheduled_pub_date -> Nullable<Date>,
        actual_pub_date -> Nullable<Date>,
        series_id -> Nullable<Int8>,
    }
}

diesel::table! {
    books.publisher (id) {
        id -> Int8,
        #[max_length = 128]
        name -> Varchar,
    }
}

diesel::table! {
    books.publisher_keyword (publisher_id, keyword) {
        publisher_id -> Int8,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::table! {
    books.series (id) {
        id -> Int8,
        #[max_length = 256]
        name -> Nullable<Varchar>,
        #[max_length = 13]
        isbn -> Nullable<Varchar>,
    }
}

diesel::joinable!(book -> publisher (publisher_id));
diesel::joinable!(book -> series (series_id));
diesel::joinable!(publisher_keyword -> publisher (publisher_id));

diesel::allow_tables_to_appear_in_same_query!(
    book,
    publisher,
    publisher_keyword,
    series,
    );