-- Delete the test data from the `chats` table
DELETE FROM chats WHERE id IN (1, 2, 3);

-- Delete the test data from the `vehicles` table
DELETE FROM vehicles WHERE plate IN ('ABC123', 'DEF456', 'GHI789');