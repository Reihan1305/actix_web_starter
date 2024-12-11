-- Add up migration script here
CREATE TABLE IF NOT EXISTS "post" (
    id serial PRIMARY KEY,
    title char(255) not null,
    content text not null,
    create_at time default NOW(),
    updated_at time,
    user_id UUID not null,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES "user" (id) ON DELETE CASCADE
);