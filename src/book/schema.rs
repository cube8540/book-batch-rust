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
        #[max_length = 32]
        site -> Varchar,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::joinable!(book -> publisher (publisher_id));
diesel::joinable!(publisher_keyword -> publisher (publisher_id));

diesel::allow_tables_to_appear_in_same_query!(
    book,
    publisher,
    publisher_keyword,
);