use crate::book::{entity, Book, BookOriginFilterRepository, BookRepository, Node, Publisher, PublisherRepository, Site};
use diesel::PgConnection;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;
type DbConnection = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>;

const MAX_BUFFER_SIZE: usize = 100;

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
        result_set.iter().for_each(|(publisher, keyword)| {
            let id = publisher.id as u64;
            let publisher = map.entry(id)
                .or_insert_with(|| Publisher::new(id, publisher.name.clone()));

            if let Some(k) = keyword {
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
    fn get_by_isbn(&self, isbn: &[&str]) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);
        let result_set = entity::find_book_by_isbn(&mut conn, isbn);
        mapping_book_and_origin_data(result_set)
    }

    fn new_books(&self, books: &[&Book]) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);

        let new_books  = books.iter()
            .map(|book| entity::NewBookEntity::new(book))
            .collect::<Vec<entity::NewBookEntity>>();

        let new_books = new_books.chunks(MAX_BUFFER_SIZE);
        let new_books = new_books.into_iter()
            .flat_map(|ch| entity::insert_books(&mut conn, ch))
            .map(|result| {
                let book = result.to_domain();
                (&book.isbn, book)
            })
            .collect();

        let new_origins = books.iter()
            .flat_map(|book| {
                let book = new_books.get(&book.isbn).unwrap();
                entity::NewBookOriginDataEntity::new(book.id as i64, &book.origin_data)
            })
            .collect::<Vec<entity::NewBookOriginDataEntity>>()
            .chunks(MAX_BUFFER_SIZE);

        new_origins.for_each(|ch| entity::insert_book_origins(&mut conn, ch));
        new_books.into_values().collect()
    }

    fn update_books(&self, books: &[&Book]) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);

        let mut updated_isbn = vec![];
        books.iter().for_each(|book| {
            let form = entity::BookForm::new(book);
            let updated = entity::update_book(&mut conn, &book.isbn, &form);
            if updated > 0 {
                let id = book.id as i64;
                book.origin_data.iter().for_each(|(site, _)| {
                    entity::delete_book_origin_data(&mut conn, id, site);
                });
                let new_origins = entity::NewBookOriginDataEntity::new(id, &book.origin_data);
                entity::insert_book_origins(&mut conn, &new_origins);
                updated_isbn.push(book.isbn.as_str())
            }
        });

        self.get_by_isbn(&updated_isbn)
    }
}

type ParentId = u64;

pub struct DieselBookOriginFilterRepository {
    pool: DbPool
}

impl DieselBookOriginFilterRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool
        }
    }

    fn get_all(&self) -> Vec<(Node, Option<ParentId>)> {
        let filter_map = RefCell::new(HashMap::new());
        let mut ref_mut = filter_map.borrow_mut();

        let mut conn = get_connection(&self.pool);
        entity::find_book_origin_filter_all(&mut conn).into_iter()
            .for_each(|e| {
                let (filter, parent_id) = e.to_domain();
                ref_mut.insert(filter.id, (Rc::new(RefCell::new(filter)), parent_id));
            });

        let items: Vec<(Node, Option<ParentId>)> = ref_mut.iter_mut()
            .map(|(_, (filter, parent_id))| (filter.clone(), *parent_id))
            .collect();

        for (filter, parent_id) in items.iter() {
            if let Some((parent, _)) = parent_id.and_then(|pid| ref_mut.get(&pid)) {
                parent.borrow_mut().add_node(filter.clone());
            }
        }

        items
    }
}

impl BookOriginFilterRepository for DieselBookOriginFilterRepository {
    fn get_root_filters(&self) -> HashMap<Site, Node> {
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

fn mapping_book_and_origin_data(entities: Vec<entity::BookWithOriginData>) -> Vec<Book> {
    let mut books = HashMap::new();
    for (book_entity, origin) in entities {
        let entry = books.entry(book_entity.isbn.clone());
        let book = entry.or_insert_with(|| book_entity.to_domain());
        if let Some(origin) = origin {
            if let Some(val) = origin.val {
                book.add_origin_data(origin.site, origin.property, val);
            }
        }
    }
    books.into_values().collect()
}
