-- down.sql
-- Step 1: Change chat_ids column back to chat_id with BIGINT type
ALTER TABLE vehicles RENAME COLUMN chat_ids TO chat_id;

ALTER TABLE vehicles ALTER COLUMN chat_id TYPE BIGINT;

ALTER TABLE vehicles DROP COLUMN found_at;

-- Step 2: Re-add the foreign key constraint
ALTER TABLE vehicles
ADD CONSTRAINT fk_chats FOREIGN KEY (chat_id) REFERENCES chats (id);

ALTER TABLE chats ADD COLUMN selected_vehicles TEXT;