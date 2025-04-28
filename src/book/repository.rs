use crate::book::{entity, Book, BookOriginFilter, BookOriginFilterRepository, BookRepository, Publisher, PublisherRepository, Site};
use chrono::Utc;
use diesel::PgConnection;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;
type DbConnection = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>;

fn get_connection(pool: &DbPool) -> DbConnection {
    pool.get().expect("Failed to get db connection from pool")
}

pub struct DieselPublisherRepository {
    pool: DbPool
}

impl DieselPublisherRepository {
    pub fn new(pool: DbPool) -> Self {
        DieselPublisherRepository {
            pool
        }
    }
}

impl PublisherRepository for DieselPublisherRepository {
    fn get_all(&self) -> Vec<Publisher> {
        let mut conn = get_connection(&self.pool);
        let result_set = entity::find_publisher_all(&mut conn);

        let mut map = HashMap::<u64, Publisher>::new();
        result_set.iter().for_each(|item| {
            let publisher_entity = &item.0;
            let keyword_entity = &item.1;

            let id = publisher_entity.id as u64;
            let publisher = map.entry(id)
                .or_insert_with(|| Publisher::new(id, publisher_entity.name.clone()));

            if let Some(k) = keyword_entity {
                publisher.add_keyword(k.site.clone(), k.keyword.clone());
            }
        });
        map.into_values().collect()
    }
}

pub struct DieselBookRepository {
    pool: DbPool
}

impl DieselBookRepository {
    pub fn new(pool: DbPool) -> Self {
        DieselBookRepository {
            pool
        }
    }
}

impl BookRepository for DieselBookRepository {
    fn get_by_isbn(&self, isbn: &Vec<&str>) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);
        let result_set = entity::find_book_by_isbn(&mut conn, isbn);
        result_set.iter()
            .map(|book| book.to_domain())
            .collect()
    }

    fn new_books(&self, books: Vec<Book>) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);
        let new_entities: Vec<entity::NewBookEntity> = books
            .iter()
            .map(|book| entity::NewBookEntity {
                isbn: &book.isbn,
                title: &book.title,
                publisher_id: book.publisher_id as i64,
                scheduled_pub_date: book.scheduled_pub_date,
                actual_pub_date: book.actual_pub_date,
                registered_at: Utc::now().naive_utc(),
            })
            .collect();

        let results = entity::insert_books(&mut conn, new_entities);
        results.into_iter().map(|result| result.to_domain()).collect()
    }

    fn update_books(&self, books: Vec<Book>) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);
        let map = books.into_iter()
            .map(|b| (b.isbn.clone(), b))
            .collect::<HashMap<String, Book>>();

        map.iter()
            .for_each(|(isbn, book)| {
                let form = entity::BookForm {
                    title: &book.title,
                    scheduled_pub_date: book.scheduled_pub_date.as_ref(),
                    actual_pub_date: book.actual_pub_date.as_ref(),
                };
                entity::update_book(&mut conn, &isbn, form);
            });

        let k = map.keys()
            .map(|k| k.as_str())
            .collect::<Vec<&str>>();
        self.get_by_isbn(&k)
    }
}

type ParentId = u64;
type BookOriginFilterRef = Rc<RefCell<BookOriginFilter>>;

pub struct DieselBookOriginFilterRepository {
    pool: DbPool
}

impl DieselBookOriginFilterRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool
        }
    }

    fn get_all(&self) -> Vec<(BookOriginFilterRef, Option<ParentId>)> {
        let filter_map = RefCell::new(HashMap::new());
        let mut ref_mut = filter_map.borrow_mut();

        let mut conn = get_connection(&self.pool);
        entity::find_book_origin_filter_all(&mut conn).into_iter()
            .for_each(|e| {
                let (filter, parent_id) = e.to_domain();
                ref_mut.insert(filter.id, (Rc::new(RefCell::new(filter)), parent_id));
            });

        let items: Vec<(BookOriginFilterRef, Option<ParentId>)> = ref_mut.iter_mut()
            .map(|(_, (filter, parent_id))| (filter.clone(), *parent_id))
            .collect();

        for (filter, parent_id) in items.iter() {
            if let Some((parent, _)) = parent_id.and_then(|pid| ref_mut.get(&pid)) {
                parent.borrow_mut().add_child(filter.clone());
            }
        }

        items
    }
}

impl BookOriginFilterRepository for DieselBookOriginFilterRepository {
    fn get_root_filters(&self) -> HashMap<Site, BookOriginFilterRef> {
        let mut map = HashMap::new();
        self.get_all().into_iter()
            .for_each(|(filter, _)| {
                if filter.borrow().is_root {
                    map.insert(filter.borrow().site.clone(), filter.clone());
                }
            });
        map
    }
}