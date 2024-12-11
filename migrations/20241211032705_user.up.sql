-- Add up migration script here
create extension if not EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS "user" (
    id UUID PRIMARY KEY NOT null default(uuid_generate_v4()),
    email CHAR(255) NOT NULL UNIQUE,
    password text not null
);