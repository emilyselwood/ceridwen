// @generated automatically by Diesel CLI.

diesel::table! {
    page (id) {
        id -> Int8,
        url -> Varchar,
        title -> Text,
        description -> Text,
        last_updated -> Timestamp,
    }
}

diesel::table! {
    word (text, page_id) {
        text -> Varchar,
        page_id -> Int8,
        count -> Int4,
    }
}

diesel::joinable!(word -> page (page_id));

diesel::allow_tables_to_appear_in_same_query!(page, word,);
