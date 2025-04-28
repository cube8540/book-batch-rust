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
        registered_at -> Timestamp,
        modified_at -> Nullable<Timestamp>,
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
    books.publisher_keyword (publisher_id, site, keyword) {
        publisher_id -> Int8,
        #[max_length = 32]
        site -> Varchar,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::table! {
    books.book_origin_filter (id) {
        id -> Int8,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 32]
        site -> Varchar,
        is_root -> Bool,
        #[max_length = 32]
        operator_type -> Nullable<Varchar>,
        #[max_length = 32]
        property_name -> Nullable<Varchar>,
        #[max_length = 256]
        regex -> Nullable<Varchar>,
        parent_id -> Nullable<Int8>,
    }
}

diesel::joinable!(book -> publisher (publisher_id));
diesel::joinable!(publisher_keyword -> publisher (publisher_id));

diesel::allow_tables_to_appear_in_same_query!(
    book,
    publisher,
    publisher_keyword,
);