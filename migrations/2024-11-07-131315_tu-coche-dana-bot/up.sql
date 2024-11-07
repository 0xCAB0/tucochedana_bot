-- SQLBook: Code

-- Your SQL goes here

-- VINTED BOT TABLES

CREATE TYPE client_state AS ENUM('initial', 'add_vehicle');

-- for trigram index

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE chats (
    id BIGINT PRIMARY KEY,
    user_id BYTEA NOT NULL,
    username TEXT NOT NULL,
    state client_state DEFAULT 'initial' NOT NULL,
    active BOOLEAN DEFAULT FALSE,
    selected_text VARCHAR(80),
    selected_vehicles TEXT,
    language_code VARCHAR(3)
);

-- Add more profiles in future countries_ids etc ...

-- IMPORTANT:
-- UPDATE duplicate_profile query if this table is updated.

CREATE TABLE vehicles (
    plate VARCHAR,
    chat_id BIGINT,
    PRIMARY KEY (plate),
    CONSTRAINT fk_chats FOREIGN KEY (chat_id) REFERENCES chats (id)
);