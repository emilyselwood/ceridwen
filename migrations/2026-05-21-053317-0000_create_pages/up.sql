-- Your SQL goes here


CREATE TABLE IF NOT EXISTS page (
    id BIGSERIAL PRIMARY KEY,
    url VARCHAR NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    last_updated TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

CREATE INDEX ix_page_url ON page (url);
