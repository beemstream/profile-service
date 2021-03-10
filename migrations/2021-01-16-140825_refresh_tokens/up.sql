-- Your SQL goes here
CREATE TABLE refresh_tokens (
    id SERIAL PRIMARY KEY,
    token VARCHAR NOT NULL,
    expiry TIMESTAMP NOT NULL,
    user_id SERIAL NOT NULL references users(id)
)
