use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use crate::data;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::index::postgres::schema::page)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Page {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: String,
    pub last_updated: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::index::postgres::schema::page)]
pub struct NewPage<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub last_updated: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::index::postgres::schema::word)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Word {
    pub text: String,
    pub page_id: i64,
    pub count: i32,
}

impl Into<data::SearchResult> for Page {
    fn into(self) -> data::SearchResult {
        data::SearchResult {
            url: self.url,
            title: self.title,
            description: self.description,
            last_index: self.last_updated.and_utc(),
        }
    }
}

impl Into<data::SearchResult> for &Page {
    fn into(self) -> data::SearchResult {
        data::SearchResult {
            url: self.url.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            last_index: self.last_updated.and_utc(),
        }
    }
}

impl<'a> From<&'a data::Page> for NewPage<'a> {
    fn from(value: &'a data::Page) -> Self {
        NewPage {
            url: &value.url.as_str(),
            title: &value.title,
            description: &value.content,
            last_updated: Utc::now().naive_utc(),
        }
    }
}
