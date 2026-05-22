use crate::{config::Config, data, error::Error, index::Index};

use diesel::prelude::*;

pub mod models;
pub mod schema;

pub struct PostgresIndex {
    conn: PgConnection,
}

impl Index<i64> for PostgresIndex {
    fn connect(config: &Config) -> Result<Self, Error> {
        Ok(Self {
            conn: PgConnection::establish(&config.database.url)?,
        })
    }

    fn search(&mut self, search_string: &str) -> Result<Vec<data::SearchResult>, Error> {
        todo!()
    }

    fn look_up_page(
        &mut self,
        page: &data::Page,
    ) -> Result<Option<(i64, data::SearchResult)>, Error> {
        let records = schema::page::dsl::page
            .filter(schema::page::url.eq(page.url.as_str()))
            .limit(1)
            .select(models::Page::as_select())
            .load(&mut self.conn)?;

        if records.is_empty() {
            return Ok(None);
        }

        let record = records.first().unwrap();
        let result: data::SearchResult = record.into();
        Ok(Some((record.id, result)))
    }

    fn lookup_id(&mut self, id: i64) -> Result<Option<data::SearchResult>, Error> {
        let records = schema::page::dsl::page
            .filter(schema::page::id.eq(id))
            .limit(1)
            .select(models::Page::as_select())
            .load(&mut self.conn)?;

        if records.is_empty() {
            return Ok(None);
        }

        let record = records.first().unwrap();
        let result: data::SearchResult = record.into();
        Ok(Some(result))
    }

    fn store_page(&mut self, page_obj: &data::Page) -> Result<(i64, data::SearchResult), Error> {
        let data: models::NewPage = page_obj.into();

        let result_post = diesel::insert_into(schema::page::table)
            .values(&data)
            .returning(models::Page::as_returning())
            .get_result(&mut self.conn)?;

        Ok((result_post.id, result_post.into()))
    }

    fn store_words(&mut self, page_id: i64, words: Vec<(String, u64)>) -> Result<(), Error> {
        let records: Vec<models::Word> = words
            .iter()
            .map(|(w, c)| models::Word {
                text: w.clone(),
                page_id,
                count: *c as i32,
            })
            .collect();

        diesel::insert_into(schema::word::table)
            .values(&records)
            .execute(&mut self.conn)?;

        Ok(())
    }
}
