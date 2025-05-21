-- Your SQL goes here

create table if not exists books.publisher(
    id bigserial not null primary key,
    name varchar(128) not null
);

create table if not exists books.series(
    id bigserial not null primary key,
    name varchar(512),
    isbn varchar(13) unique,
    main_title_vec vector(1024),
    registered_at timestamp not null default now(),
    modified_at timestamp
);

create table if not exists books.book(
    id bigserial not null primary key,
    title varchar(512) not null ,
    series_id bigserial,
    publisher_id bigserial not null,
    scheduled_pub_date date,
    actual_pub_date date,
    registered_at timestamp not null default now(),
    modified_at timestamp,

    foreign key(publisher_id) references books.publisher(id),
    foreign key(series_id) references books.series(id)
);

create table if not exists books.book_origin_filter(
    id bigserial not null primary key,
    name varchar(64) not null,
    site varchar(32) not null,
    is_root boolean not null,
    operator_type varchar(32),
    property_name varchar(32),
    regex varchar(256),
    parent_id bigserial,

    foreign key (parent_id) references books.book_origin_filter(id)
);