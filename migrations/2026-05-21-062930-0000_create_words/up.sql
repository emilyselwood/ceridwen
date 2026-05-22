-- Your SQL goes here
CREATE TABLE IF NOT EXISTS word (
    text VARCHAR NOT NULL,
    page_id BIGINT NOT NULL REFERENCES page (id),
    count INT NOT NULL,
    PRIMARY KEY (text, page_id)
);

CREATE INDEX ix_word_text ON word (text);
CREATE INDEX ix_word_page_id ON word (page_id);
