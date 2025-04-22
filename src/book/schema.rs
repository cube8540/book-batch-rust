diesel::table! {
    books.publisher (id) {
        id -> BigInt,
        name -> Varchar,
    }
}

diesel::table! {
    books.publisher_keyword (publisher_id, keyword) {
        publisher_id -> BigInt,
        keyword -> Varchar
    }
}

diesel::table! {
    books.series (id) {
        id -> BigInt,
        name -> Varchar,
        isbn -> Varchar
    }
}

diesel::table! {
    books.book (id) {
        id -> BigInt,
        isbn -> Varchar,
        title -> Varchar,
        publisher_id -> BigInt,
        scheduled_pub_date -> Nullable<Date>,
        actual_pub_date -> Nullable<Date>,
        series_id -> Nullable<BigInt>
    }
}

diesel::joinable!(publisher_keyword -> publisher (publisher_id));
diesel::allow_tables_to_appear_in_same_query!(
    publisher,
    publisher_keyword,
);

diesel::joinable!(book -> publisher (publisher_id));
diesel::allow_tables_to_appear_in_same_query!(
    book,
    publisher,
);

diesel::joinable!(book -> series (series_id));
diesel::allow_tables_to_appear_in_same_query!(
    book,
    series,
);