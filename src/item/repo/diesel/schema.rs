// @generated automatically by Diesel CLI.

pub mod books {
    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;

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
            registered_at -> Timestamp,
            modified_at -> Nullable<Timestamp>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;

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
            #[sql_name = "regex"]
            regex_val -> Nullable<Varchar>,
            parent_id -> Nullable<Int8>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;

        books.publisher (id) {
            id -> Int8,
            #[max_length = 128]
            name -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;

        books.publisher_keyword (publisher_id, site, keyword) {
            publisher_id -> Int8,
            #[max_length = 32]
            site -> Varchar,
            #[max_length = 32]
            keyword -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;

        books.series (id) {
            id -> Int8,
            #[max_length = 256]
            name -> Nullable<Varchar>,
            #[max_length = 13]
            isbn -> Nullable<Varchar>,
            vec -> Nullable<Vector>,
            registered_at -> Timestamp,
            modified_at -> Nullable<Timestamp>,
        }
    }

    diesel::joinable!(book -> publisher (publisher_id));
    diesel::joinable!(book -> series (series_id));
    diesel::joinable!(publisher_keyword -> publisher (publisher_id));

    diesel::allow_tables_to_appear_in_same_query!(
        book,
        book_origin_filter,
        publisher,
        publisher_keyword,
        series,
    );
}
