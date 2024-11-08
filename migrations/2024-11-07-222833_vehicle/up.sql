-- Your SQL goes here up.sql
-- Step 1: Drop the foreign key constraint
ALTER TABLE vehicles DROP CONSTRAINT fk_chats;

-- Step 2: Change the chat_id column to chat_ids with VARCHAR type
ALTER TABLE vehicles RENAME COLUMN chat_id TO chat_ids;

ALTER TABLE vehicles ALTER COLUMN chat_ids TYPE VARCHAR;

ALTER TABLE vehicles ADD COLUMN found_at TIMESTAMP WITH TIME ZONE;

ALTER TABLE chats DROP COLUMN selected_vehicles;