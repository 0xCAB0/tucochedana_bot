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
    subscribed_vehicles TEXT,
    language_code VARCHAR(3)
);

CREATE TABLE vehicles (
    plate VARCHAR,
    subscribers_ids TEXT,
    found_at TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (plate)
);