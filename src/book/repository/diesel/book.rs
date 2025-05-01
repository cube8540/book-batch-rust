use crate::book;
use crate::book::repository::diesel::{entity, get_connection, schema, sql_debugging, DbPool};
use crate::book::repository::{BookRepository, SQLError};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;
use tracing::error;

pub struct Repository {
    pool: DbPool
}

pub fn new(pool: DbPool) -> Repository {
    Repository { pool }
}

impl BookRepository for Repository {
    fn find_by_isbn<'book, I>(&self, isbn: I) -> Vec<book::Book>
    where
        I: Iterator<Item=&'book str>
    {
        let entities: Vec<entity::Book> = sql_debugging(schema::book::table
            .filter(schema::book::isbn.eq_any(isbn))
            .select(entity::Book::as_select()))
            .into_boxed()
            .load(&mut get_connection(&self.pool))
            .unwrap();

        entities.iter()
            .map(|e| e.to_domain())
            .collect()
    }

    fn find_origin_by_id<I>(&self, id: I) -> HashMap<u64, HashMap<book::Site, book::Original>>
    where
        I: Iterator<Item=u64>
    {
        let mut result: HashMap<u64, HashMap<book::Site, book::Original>> = HashMap::new();
        let id = id.map(|id| id.clone() as i64);

        let origins: Vec<entity::BookOriginData> = sql_debugging(schema::book_origin_data::table
            .filter(schema::book_origin_data::book_id.eq_any(id))
            .select(entity::BookOriginData::as_select()))
            .load(&mut get_connection(&self.pool))
            .unwrap();

        origins.into_iter().for_each(|entity| {
            if let Some(v) = entity.val {
                let id = entity.book_id as u64;
                result.entry(id)
                    .or_insert_with(|| HashMap::new()).entry(entity.site)
                    .or_insert_with(|| HashMap::new())
                    .insert(entity.property, v);
            }
        });
        result
    }

    fn new_books<'book, I>(&self, books: I, with_origin: bool) -> Result<Vec<book::Book>, SQLError>
    where
        I: IntoIterator<Item=&'book book::Book>
    {
        let mut mapping_books: HashMap<String, &book::Book> = HashMap::new();
        let mut new_books: Vec<entity::NewBook> = vec![];

        for book in books {
            mapping_books.insert(book.isbn.clone(), book);
            new_books.push(entity::NewBook::new(book));
        }

        let mut connection = get_connection(&self.pool);
        let registered_books: Result<Vec<book::Book>, SQLError> = sql_debugging(diesel::insert_into(schema::book::table)
            .values(new_books))
            .get_results::<entity::Book>(&mut connection)
            .map(|entities| entities.into_iter().map(|e| e.to_domain()).collect())
            .map_err(|err| SQLError::QueryExecuteError(err.to_string()));

        if let Ok(registered_books) = registered_books.as_ref() {
            if with_origin {
                let new_origins = registered_books.iter()
                    .filter_map(|registered_book| {
                        mapping_books.get(&registered_book.isbn)
                            .map(|b| (registered_book.id, &b.origin_data))
                    });
                if let Err(err) = self.new_origin_data(new_origins) {
                    return Err(SQLError::QueryExecuteError(format!("도서 원본 데이터 저장중 에러가 발생 했습니다. 원본 데이터를 제외한 도서들은 모두 정상적으로 저장 되었습니다. => {:?}", err)));
                }
            }
        }
        registered_books
    }

    fn new_origin_data<'book, I>(&self, origins: I) -> Result<usize, SQLError>
    where
        I: IntoIterator<Item=(u64, &'book HashMap<book::Site, book::Original>)>
    {
        let new_origins: Vec<entity::NewBookOriginDataEntity> = origins.into_iter()
            .flat_map(|(id, original)| {
                original.iter()
                    .flat_map(move |(key, val)| {
                        entity::NewBookOriginDataEntity::new(id as i64, key, val)
                    })
            })
            .collect();

        sql_debugging(diesel::insert_into(schema::book_origin_data::table)
            .values(new_origins))
            .execute(&mut get_connection(&self.pool))
            .map_err(|err| SQLError::QueryExecuteError(err.to_string()))
    }

    fn update_book(&self, book: &book::Book, with_origin: bool) -> Result<usize, SQLError> {
        let mut updated_count = sql_debugging(diesel::update(schema::book::table)
            .filter(schema::book::id.eq(book.id as i64))
            .set(entity::BookForm::new(book)))
            .execute(&mut get_connection(&self.pool))
            .map_err(|err| SQLError::QueryExecuteError(err.to_string()))?;

        if updated_count > 0 && with_origin {
            let mut delete_successes: Vec<&book::Site> = vec![];
            book.origin_data.iter().for_each(|(site, _)| {
                if self.delete_origin_data(book.id, site).is_ok() {
                    delete_successes.push(site);
                } else {
                    error!("원본 데이터 삭제 중 에러가 발생 하였습니다. => {:?} (ISBN: {})", site, book.isbn);
                }
            });
            let new_origin_data = book.origin_data.iter()
                .filter(|(site, _)| delete_successes.contains(site))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<HashMap<book::Site, book::Original>>();

            updated_count += self.new_origin_data([(book.id, &new_origin_data)])
                .unwrap_or_else(|err| {
                    error!("원본 데이터 저장 중 에러가 발생 하였습니다. 원본을 제외한 도서 정보는 정상적으로 저장 되었습니다. => {:?} (ISBN: {})", err, book.isbn);
                    0
                });
        }
        Ok(updated_count)
    }

    fn delete_origin_data(&self, id: u64, site: &book::Site) -> Result<usize, SQLError> {
        sql_debugging(diesel::delete(schema::book_origin_data::dsl::book_origin_data
            .filter(
                schema::book_origin_data::book_id.eq(id as i64)
                    .and(schema::book_origin_data::site.eq(site))
            )))
            .into_boxed()
            .execute(&mut get_connection(&self.pool))
            .map_err(|err| SQLError::QueryExecuteError(err.to_string()))
    }
}