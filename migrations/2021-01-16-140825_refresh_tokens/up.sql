-- Your SQL goes here
CREATE TABLE refresh_tokens (
    id SERIAL PRIMARY KEY,
    expiry TIMESTAMP NOT NULL
)
