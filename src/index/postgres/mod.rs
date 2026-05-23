use crate::{
    config::Config,
    data,
    error::Error,
    index::Index,
    utils::text_tools::{filter, tokenise},
};

use diesel::{dsl::sum, prelude::*};

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
        // Format and filter the search string into a bunch of values

        let words = filter(tokenise(search_string));

        // create query
        let query = schema::page::dsl::page
            .inner_join(schema::word::dsl::word)
            .select(schema::page::all_columns)
            .filter(schema::word::text.eq_any(words))
            .group_by(schema::page::all_columns)
            .order_by(sum(schema::word::count).desc());

        //debug!("final query: {}", query.into_sql());

        let result = query.load(&mut self.conn)?;

        // Convert the result of the query into a vec of search results.
        Ok(result.iter().map(|p: &models::Page| p.into()).collect())
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
            .filter(|(w, _c)| w.len() < 1000)
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
