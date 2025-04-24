-- Your SQL goes here
create sequence if not exists books.publisher_id_seq;
create table if not exists books.publisher (
                                               id bigint not null primary key default nextval('books.publisher_id_seq'),
                                               name varchar(128) not null
);
alter sequence if exists books.publisher_id_seq owned by books.publisher.id;

create table books.publisher_keyword (
                                   publisher_id bigint not null ,
                                   site varchar(32) not null ,
                                   keyword varchar(256) not null ,
                                   primary key (publisher_id, site, keyword),
                                   foreign key (publisher_id) references books.publisher(id)
);

create sequence if not exists books.series_id_seq;
create table if not exists books.series(
                                           id bigint not null primary key default nextval('books.series_id_seq'),
                                           name varchar(256),
                                           isbn varchar(13) unique
);
alter sequence if exists books.series_id_seq owned by books.series.id;

create sequence if not exists books.book_id_seq;
create table if not exists books.book(
                                         id bigint not null primary key default nextval('books.book_id_seq'),
                                         isbn varchar(13) not null unique ,
                                         title varchar(256) not null ,
                                         publisher_id bigint not null ,
                                         scheduled_pub_date date ,
                                         actual_pub_date date,
                                         series_id bigint,

                                         foreign key (publisher_id) references books.publisher(id),
                                         foreign key (series_id) references books.series(id)
);
alter sequence if exists books.book_id_seq owned by books.book.id;