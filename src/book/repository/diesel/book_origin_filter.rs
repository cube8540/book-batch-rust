use crate::book::repository::diesel::entity::BookOriginFilterEntity;
use crate::book::repository::diesel::{get_connection, schema, DbPool};
use crate::book::{BookOriginFilterRepository, Node, Site};

use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Repository {
    pool: DbPool
}

type ParentId = u64;
impl Repository {

    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl BookOriginFilterRepository for Repository {
    fn get_root_filters(&self) -> HashMap<Site, Node> {
        let mut result = HashMap::new();

        let filter_map = RefCell::new(HashMap::new());
        let mut ref_mut = filter_map.borrow_mut();

        let filters: Vec<BookOriginFilterEntity> = schema::book_origin_filter::table
            .select(BookOriginFilterEntity::as_select())
            .load(&mut get_connection(&self.pool))
            .unwrap();

        filters.into_iter().for_each(|e| {
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

        items.into_iter()
            .for_each(|(filter, _)| {
                if filter.borrow().is_root {
                    result.insert(filter.borrow().site.clone(), filter.clone());
                }
            });
        result
    }
}