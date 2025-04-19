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

diesel::joinable!(publisher_keyword -> publisher (publisher_id));
diesel::allow_tables_to_appear_in_same_query!(
    publisher,
    publisher_keyword,
);
